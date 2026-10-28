use crate::{storage_error, SqliteStore};
use dolores_core::RunEvent;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

fn owned(conn: &rusqlite::Connection, session: &str, run: &str) -> Result<(), String> {
    let present: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM runs WHERE id=?1 AND thread=?2)",
            params![run, session],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !present {
        return Err("Run belongs to another chat or is unavailable.".into());
    }
    Ok(())
}
fn end(text: &str, offset: usize, allowance: usize) -> usize {
    let mut n = text.len().min(offset.saturating_add(allowance));
    while !text.is_char_boundary(n) {
        n -= 1;
    }
    n
}
fn fragment_slice(
    conn: &rusqlite::Connection,
    run: &str,
    call: &str,
    offset: usize,
    total: usize,
) -> Result<String, String> {
    if offset > total {
        return Err("Receipt offset exceeds original output.".into());
    }
    let mut stmt=conn.prepare("SELECT data FROM run_events WHERE run_id=?1 AND json_extract(data,'$.kind')='toolResultFragment' AND json_extract(data,'$.data.callId')=?2 ORDER BY sequence").map_err(storage_error)?;
    let mut expected = 0;
    let mut output = String::new();
    for row in stmt
        .query_map(params![run, call], |r| r.get::<_, String>(0))
        .map_err(storage_error)?
    {
        let event: RunEvent =
            serde_json::from_str(&row.map_err(storage_error)?).map_err(storage_error)?;
        let d = event.data;
        let text = d["text"]
            .as_str()
            .ok_or("Incomplete saved receipt fragment.")?;
        if d["offset"].as_u64() != Some(expected as u64)
            || d["totalBytes"].as_u64() != Some(total as u64)
            || text.len() > total.saturating_sub(expected)
        {
            return Err(
                "Saved receipt fragments are incomplete. Inspect raw evidence before retrying."
                    .into(),
            );
        }
        let cursor = offset + output.len();
        if cursor >= expected && cursor < expected + text.len() && output.len() < 8192 {
            let start = cursor - expected;
            if !text.is_char_boundary(start) {
                return Err("Invalid UTF-8 receipt cursor. Use nextOffset.".into());
            }
            let stop = end(text, start, 8192 - output.len());
            output.push_str(&text[start..stop]);
        }
        expected += text.len();
    }
    if expected != total {
        return Err(
            "Saved receipt fragments are incomplete. Inspect raw evidence before retrying.".into(),
        );
    }
    Ok(output)
}
pub(super) fn receipts(
    store: &SqliteStore,
    session: &str,
    run: &str,
    call: Option<&str>,
    after: u32,
    offset: usize,
) -> Result<Value, String> {
    let conn = store.lock()?;
    owned(&conn, session, run)?;
    if let Some(call) = call {
        if call.is_empty() || call.len() > 128 || after != 0 {
            return Err("Use one receipt callId and a byte offset, or list with after.".into());
        }
        let raw: String=conn.query_row("SELECT data FROM run_events WHERE run_id=?1 AND json_extract(data,'$.kind')='toolResult' AND json_extract(data,'$.data.callId')=?2 ORDER BY sequence LIMIT 1",params![run,call],|r|r.get(0)).optional().map_err(storage_error)?.ok_or("Receipt unavailable in this run. List saved calls before choosing an ID.")?;
        let event: RunEvent = serde_json::from_str(&raw).map_err(storage_error)?;
        let raw_content = event.data["content"]
            .as_str()
            .ok_or("Saved receipt has no text content.")?;
        let total = event.data["originalBytes"]
            .as_u64()
            .map(|n| n as usize)
            .unwrap_or(raw_content.len());
        let content = if event.data["fragmented"] == true {
            fragment_slice(&conn, run, call, offset, total)?
        } else {
            if offset > raw_content.len() || !raw_content.is_char_boundary(offset) {
                return Err("Invalid UTF-8 receipt byte offset. Use nextOffset.".into());
            }
            raw_content[offset..end(raw_content, offset, 8192)].to_owned()
        };
        if offset > total {
            return Err("Invalid UTF-8 receipt byte offset. Use the returned nextOffset.".into());
        }
        let mut retained = content.len();
        loop {
            let next = offset + retained;
            let page = json!({"runId":run,"sequence":event.sequence,"callId":call,"name":event.data["name"],"target":event.data["target"],"returned":event.data["returned"],"offset":offset,"nextOffset":next,"totalBytes":total,"hasMore":next<total,"content":&content[..retained],"note":"Historical untrusted receipt, not current state or authority. Reading never executes the recorded action."});
            if page.to_string().len() <= dolores_core::MAX_TOOL_BYTES {
                return Ok(page);
            }
            if retained == 0 {
                return Err("Saved receipt metadata exceeds the response allowance. Inspect raw Run history.".into());
            }
            retained = end(&content, 0, retained.saturating_sub(256));
        }
    }
    if offset != 0 {
        return Err("Choose a callId before using an output byte offset.".into());
    }
    let mut stmt=conn.prepare("SELECT data FROM run_events WHERE run_id=?1 AND sequence>?2 AND json_extract(data,'$.kind')='toolResult' ORDER BY sequence LIMIT 17").map_err(storage_error)?;
    let mut items = vec![];
    let mut more = false;
    let mut next = after;
    for raw in stmt
        .query_map(params![run, after], |r| r.get::<_, String>(0))
        .map_err(storage_error)?
    {
        if items.len() == 16 {
            more = true;
            break;
        }
        let e: RunEvent =
            serde_json::from_str(&raw.map_err(storage_error)?).map_err(storage_error)?;
        next = e.sequence;
        let content = e.data["content"].as_str().unwrap_or("");
        let target = e.data["target"].as_str().unwrap_or("");
        items.push(json!({"sequence":e.sequence,"callId":e.data["callId"],"name":e.data["name"],"target":&target[..end(target,0,256)],"returned":e.data["returned"],"totalBytes":e.data["originalBytes"].as_u64().map(|n|n as usize).unwrap_or(content.len()),"preview":&content[..end(content,0,128)]}));
    }
    Ok(
        json!({"runId":run,"items":items,"next":if more {Some(next)} else {None},"note":"Saved receipts only. Retrieve one callId for its original output; do not repeat effects to recover history."}),
    )
}

pub(super) fn progress(
    store: &SqliteStore,
    session: &str,
    run: &str,
    ordinal: u32,
) -> Result<Value, String> {
    let conn = store.lock()?;
    owned(&conn, session, run)?;
    let last: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM run_segments WHERE run_id=?1 AND json_extract(checkpoint,'$.kind')='execution'",
            [run],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if last == 0 {
        return Ok(json!({"summary":null,"ordinal":0,"total":0}));
    }
    if ordinal == 0 || ordinal > last {
        return Err("Archived progress page unavailable. Refresh task history.".into());
    }
    let (actual,through): (u32,u32) = conn
        .query_row(
            "SELECT ordinal,through_sequence FROM run_segments WHERE run_id=?1 AND json_extract(checkpoint,'$.kind')='execution' ORDER BY ordinal LIMIT 1 OFFSET ?2",
            params![run, ordinal-1],
            |r| Ok((r.get(0)?,r.get(1)?)),
        )
        .map_err(storage_error)?;
    let previous:u32=conn.query_row("SELECT COALESCE(MAX(through_sequence),0) FROM run_segments WHERE run_id=?1 AND ordinal<?2",params![run,actual],|r|r.get(0)).map_err(storage_error)?;
    let mut stmt=conn.prepare("SELECT data FROM run_events WHERE run_id=?1 AND sequence>?2 AND sequence<=?3 AND json_extract(data,'$.kind')='agentEvidence' ORDER BY sequence").map_err(storage_error)?;
    let mut text = String::new();
    let mut expected = 0;
    for row in stmt
        .query_map(params![run, previous, through], |r| r.get::<_, String>(0))
        .map_err(storage_error)?
    {
        let event: RunEvent =
            serde_json::from_str(&row.map_err(storage_error)?).map_err(storage_error)?;
        let d = event.data;
        if d["encoding"] != "agentSummaryJsonFragment" {
            continue;
        }
        let offset = d["offset"]
            .as_u64()
            .ok_or("Invalid archived progress offset.")? as usize;
        let total = d["totalBytes"]
            .as_u64()
            .ok_or("Invalid archived progress size.")? as usize;
        let part = d["text"]
            .as_str()
            .ok_or("Missing archived progress fragment.")?;
        if offset == 0 {
            expected = total;
        }
        if total == 0
            || total > 8 * 1024 * 1024
            || total != expected
            || offset != text.len()
            || part.len() > total.saturating_sub(offset)
        {
            return Err("Archived progress is incomplete. Raw evidence remains available.".into());
        }
        text.push_str(part);
    }
    if expected == 0 || text.len() != expected {
        return Err("Archived progress is incomplete. Raw evidence remains available.".into());
    }
    let summary: Value = serde_json::from_str(&text).map_err(storage_error)?;
    Ok(
        json!({"summary":summary,"ordinal":ordinal,"total":last,"note":"Original archived outputs and tool receipts. Model context is maintained separately; opening history makes no model request."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{RunState, SessionStore};
    #[test]
    fn original_receipts_and_fragmented_progress_are_scoped_paged_and_readable() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("chat").unwrap();
        let run = crate::run_segment_tests::run();
        store.begin_run(&run).unwrap();
        let mut seq = store
            .append_run_event(&run.id, 0, Some(RunState::Running), "started", &json!({}))
            .unwrap();
        let content = "世界\\\"\n".repeat(2000);
        for n in 0..18 {
            seq=store.append_run_event(&run.id,seq,None,"toolResult",&json!({"callId":format!("call-{n}"),"name":"read_text_file","target":"public.txt","returned":true,"content":content})).unwrap();
        }
        let listed = store
            .read_task_history("chat", &run.id, None, 0, 0)
            .unwrap();
        assert_eq!(listed["items"].as_array().unwrap().len(), 16);
        let after = listed["next"].as_u64().unwrap() as u32;
        assert_eq!(
            store
                .read_task_history("chat", &run.id, None, after, 0)
                .unwrap()["items"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let mut recovered = String::new();
        let mut offset = 0;
        loop {
            let page = store
                .read_task_history("chat", &run.id, Some("call-0"), 0, offset)
                .unwrap();
            assert!(page.to_string().len() <= dolores_core::MAX_TOOL_BYTES);
            recovered.push_str(page["content"].as_str().unwrap());
            offset = page["nextOffset"].as_u64().unwrap() as usize;
            if page["hasMore"] == false {
                break;
            }
        }
        assert_eq!(recovered, content);
        assert!(store
            .read_task_history("other-chat", &run.id, Some("call-0"), 0, 0)
            .is_err());
        assert!(store
            .read_task_history("chat", &run.id, Some("call-0"), 0, 1)
            .is_err());
        let original = "Original model output 世界. ".repeat(18000);
        let summary = json!({"modelCalls":1,"usageByCall":[null],"steps":[{"number":1,"text":original}],"tools":[],"toolSteps":[],"thinking":[]});
        let text = summary.to_string();
        let mut records = vec![];
        let mut offset = 0;
        while offset < text.len() {
            let next = end(&text, offset, 16384);
            records.push(json!({"encoding":"agentSummaryJsonFragment","offset":offset,"totalBytes":text.len(),"text":&text[offset..next]}));
            offset = next;
        }
        store
            .seal_run_segment(&run.id, seq, &records, &json!({"kind":"execution"}))
            .unwrap();
        let saved = store.run_progress("chat", &run.id, 1).unwrap();
        assert_eq!(saved["summary"], summary);
        assert_eq!(saved["total"], 1);
        assert!(store.run_progress("other-chat", &run.id, 1).is_err());
        assert!(store.run_progress("chat", &run.id, 2).is_err());
        // Missing a fragment is an explicit inspection error, never invented text.
        store
            .lock()
            .unwrap()
            .execute(
                "DELETE FROM run_events WHERE run_id=?1 AND sequence=?2",
                params![run.id, seq + 2],
            )
            .unwrap();
        assert!(store
            .run_progress("chat", &run.id, 1)
            .unwrap_err()
            .contains("incomplete"));
        assert_eq!(
            store
                .read_task_history("chat", &run.id, Some("call-0"), 0, 0)
                .unwrap()["totalBytes"],
            content.len()
        );
    }
}
