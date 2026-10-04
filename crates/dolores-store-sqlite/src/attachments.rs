use super::{storage_error, SqliteStore};
use dolores_core::{
    AttachmentData, AttachmentRef, MAX_ATTACHMENT_STORE_BYTES, MAX_DRAFT_ATTACHMENTS,
};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
fn parse(data: Option<String>) -> Result<Vec<AttachmentRef>, String> {
    let parts: Vec<AttachmentRef> = data
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(storage_error)?
        .unwrap_or_default();
    if parts.len() > MAX_DRAFT_ATTACHMENTS {
        return Err("Attachment reference count is invalid.".into());
    }
    for p in &parts {
        p.validate()?;
    }
    Ok(parts)
}
pub(super) fn draft(c: &Connection, id: &str) -> Result<Vec<AttachmentRef>, String> {
    let exists: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [id],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !exists {
        return Err("Chat no longer exists.".into());
    }
    parse(
        c.query_row(
            "SELECT data FROM draft_attachments WHERE session_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?,
    )
}
pub(super) fn parts(c: &Connection, id: i64) -> Result<Vec<AttachmentRef>, String> {
    parse(
        c.query_row(
            "SELECT data FROM message_parts WHERE message_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?,
    )
}
pub(super) fn commit_draft(c: &Connection, session: &str, id: i64) -> Result<(), String> {
    let parts = draft(c, session)?;
    if !parts.is_empty() {
        c.execute(
            "INSERT INTO message_parts(message_id,data) VALUES(?1,?2)",
            params![id, serde_json::to_string(&parts).map_err(storage_error)?],
        )
        .map_err(storage_error)?;
    }
    c.execute(
        "DELETE FROM draft_attachments WHERE session_id=?1",
        [session],
    )
    .map_err(storage_error)?;
    Ok(())
}
pub(super) fn cleanup(c: &Connection) -> Result<usize, String> {
    c.execute("DELETE FROM attachment_assets WHERE NOT EXISTS(SELECT 1 FROM draft_attachments,json_each(draft_attachments.data) WHERE json_extract(value,'$.digest')=attachment_assets.digest) AND NOT EXISTS(SELECT 1 FROM message_parts,json_each(message_parts.data) WHERE json_extract(value,'$.digest')=attachment_assets.digest)",[]).map_err(storage_error)
}
pub(super) fn image_models(c: &Connection, base: &str) -> Result<Vec<String>, String> {
    let data: Option<String> = c
        .query_row(
            "SELECT data FROM model_images WHERE base_url=?1",
            [base],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    data.map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(storage_error)
        .map(|v| v.unwrap_or_default())
}
pub(super) fn save_image_models(
    c: &Connection,
    base: &str,
    models: &[String],
) -> Result<(), String> {
    if models.len() > 32
        || models
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 200 || s.chars().any(char::is_control))
    {
        return Err("Image capability must refer to up to 32 configured model IDs.".into());
    }
    c.execute("INSERT INTO model_images(base_url,data) VALUES(?1,?2) ON CONFLICT(base_url) DO UPDATE SET data=excluded.data",params![base,serde_json::to_string(models).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(())
}
impl SqliteStore {
    pub(super) fn save_attachment(&self, id: &str, data: &AttachmentData) -> Result<(), String> {
        data.reference.validate()?;
        if data.reference.bytes != data.data.len()
            || format!("{:x}", Sha256::digest(&data.data)) != data.reference.digest
        {
            return Err("Attachment snapshot changed. Attach it again.".into());
        }
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut parts = draft(&tx, id)?;
        let repeated = parts.iter().any(|p| p.digest == data.reference.digest);
        if !repeated && parts.len() >= MAX_DRAFT_ATTACHMENTS {
            return Err(
                "Attach up to four files per message. Remove a file before adding another.".into(),
            );
        }
        let existing: Option<Vec<u8>> = tx
            .query_row(
                "SELECT data FROM attachment_assets WHERE digest=?1",
                [&data.reference.digest],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let stored: usize = tx
            .query_row(
                "SELECT COALESCE(SUM(length(data)),0) FROM attachment_assets",
                [],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if stored.saturating_sub(existing.as_ref().map_or(0, Vec::len)) + data.data.len()
            > MAX_ATTACHMENT_STORE_BYTES
        {
            return Err("Local attachments reached 64 MiB. Clean unused snapshots or delete unneeded chats before attaching more.".into());
        }
        tx.execute(
            "INSERT INTO attachment_assets(digest,data) VALUES(?1,?2) ON CONFLICT(digest) DO UPDATE SET data=excluded.data",
            params![data.reference.digest, data.data],
        )
        .map_err(storage_error)?;
        if !repeated {
            parts.push(data.reference.clone());
        }
        tx.execute("INSERT INTO draft_attachments(session_id,data) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET data=excluded.data",params![id,serde_json::to_string(&parts).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)
    }
    pub(super) fn drop_attachment(&self, id: &str, digest: &str) -> Result<(), String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut parts = draft(&tx, id)?;
        parts.retain(|p| p.digest != digest);
        tx.execute(
            "UPDATE draft_attachments SET data=?2 WHERE session_id=?1",
            params![id, serde_json::to_string(&parts).map_err(storage_error)?],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)
    }
    pub(super) fn read_attachment(&self, id: &str, digest: &str) -> Result<AttachmentData, String> {
        let c = self.lock()?;
        let mut refs = draft(&c, id)?;
        let mut stmt=c.prepare("SELECT p.data FROM message_parts p JOIN messages m ON m.id=p.message_id WHERE m.session_id=?1 AND EXISTS(SELECT 1 FROM json_each(p.data) WHERE json_extract(value,'$.digest')=?2) LIMIT 1").map_err(storage_error)?;
        let data: Option<String> = stmt
            .query_row(params![id, digest], |r| r.get(0))
            .optional()
            .map_err(storage_error)?;
        refs.extend(parse(data)?);
        let reference = refs
            .into_iter()
            .find(|p| p.digest == digest)
            .ok_or("Attachment is not referenced by this chat.")?;
        let data: Option<Vec<u8>> = c
            .query_row(
                "SELECT data FROM attachment_assets WHERE digest=?1",
                [digest],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let data=data.ok_or("Attachment snapshot is missing. Restore it or remove the reference/start a new chat; your draft is retained.")?;
        if data.len() != reference.bytes || format!("{:x}", Sha256::digest(&data)) != digest {
            return Err("Attachment snapshot changed. Reattach the file or start a new chat; your draft is retained.".into());
        }
        Ok(AttachmentData { reference, data })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    fn asset(name: &str, text: &str) -> AttachmentData {
        AttachmentData {
            reference: AttachmentRef {
                digest: format!("{:x}", Sha256::digest(text.as_bytes())),
                name: name.into(),
                mime: "text/plain".into(),
                bytes: text.len(),
            },
            data: text.as_bytes().to_vec(),
        }
    }
    #[test]
    fn snapshots_are_scoped_atomic_and_survive_send_fork_and_cleanup() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create("chat").unwrap();
        s.create("other").unwrap();
        let data = asset("notes.txt", "Keep 世界");
        s.add_attachment("chat", &data).unwrap();
        assert!(s.attachment_data("other", &data.reference.digest).is_err());
        assert_eq!(s.cleanup_attachments().unwrap(), 0);
        s.commit_turn("chat", "read attachment", "done").unwrap();
        assert!(s.draft_attachments("chat").unwrap().is_empty());
        let page = s.messages_page("chat", None, false, 80).unwrap();
        assert_eq!(page.items[0].parts, vec![data.reference.clone()]);
        s.fork_session("chat", page.items[1].id, "fork").unwrap();
        s.delete("chat").unwrap();
        assert_eq!(s.cleanup_attachments().unwrap(), 0);
        assert_eq!(
            s.attachment_data("fork", &data.reference.digest)
                .unwrap()
                .data,
            data.data
        );
        s.delete("fork").unwrap();
        assert_eq!(s.cleanup_attachments().unwrap(), 1);
    }
    #[test]
    fn tampered_and_excess_snapshots_keep_previous_draft() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create("chat").unwrap();
        for n in 0..4 {
            s.add_attachment("chat", &asset("note.txt", &format!("Note {n}")))
                .unwrap();
        }
        assert!(s
            .add_attachment("chat", &asset("extra.txt", "too many"))
            .is_err());
        let p = s.draft_attachments("chat").unwrap();
        assert_eq!(p.len(), 4);
        s.lock()
            .unwrap()
            .execute(
                "UPDATE attachment_assets SET data=?1 WHERE digest=?2",
                params![b"bad".as_slice(), p[0].digest],
            )
            .unwrap();
        assert!(s
            .attachment_data("chat", &p[0].digest)
            .unwrap_err()
            .contains("changed"));
        assert_eq!(s.draft_attachments("chat").unwrap(), p);
        let original = asset("note.txt", "Note 0");
        s.add_attachment("chat", &original).unwrap();
        assert_eq!(
            s.attachment_data("chat", &p[0].digest).unwrap().data,
            original.data
        );
        assert_eq!(s.draft_attachments("chat").unwrap(), p);
        s.lock()
            .unwrap()
            .execute(
                "DELETE FROM attachment_assets WHERE digest=?1",
                [&p[0].digest],
            )
            .unwrap();
        assert!(s.attachment_data("chat", &p[0].digest).is_err());
        s.add_attachment("chat", &original).unwrap();
        assert_eq!(
            s.attachment_data("chat", &p[0].digest).unwrap().data,
            original.data
        );
    }
}
