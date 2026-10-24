use super::*;
use dolores_core::attention::*;
use serde_json::{json, Value};
fn s() -> SqliteStore {
    let s = SqliteStore::open(Path::new(":memory:")).unwrap();
    s.create("task").unwrap();
    s.save_draft("task", "Keep private draft").unwrap();
    s
}
fn begin(s: &SqliteStore, id: &str) {
    s.begin_run(&serde_json::from_value(json!({"id":id,"thread":"task","model":"fixture","settings":RequestSettings::default(),"input":"Private prompt never in notification","state":"prepared","sequence":0,"createdAt":1,"build":"fixture","tools":[]})).unwrap()).unwrap();
    s.append_run_event(
        id,
        0,
        Some(dolores_core::RunState::Running),
        "started",
        &json!({}),
    )
    .unwrap();
}
fn poll(s: &SqliteStore) -> Value {
    s.attention(&Request::Poll { cursor: None }).unwrap()
}
fn enable(s: &SqliteStore) {
    let p: Preferences =
        serde_json::from_value(s.attention(&Request::Preferences).unwrap()).unwrap();
    s.attention(&Request::SavePreferences {
        preferences: Preferences { enabled: true, ..p },
    })
    .unwrap();
}
const RUN: &str = "11111111-1111-4111-8111-111111111111";
#[test]
fn actual_restart_retains_uncertain_handoff_and_migration_skips_old_notifications() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("attention.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("task").unwrap();
    enable(&store);
    begin(&store, RUN);
    store
        .append_run_event(
            RUN,
            1,
            Some(dolores_core::RunState::Completed),
            "finished",
            &json!({}),
        )
        .unwrap();
    let p = poll(&store);
    let id = p["items"][0]["id"].as_str().unwrap().to_owned();
    store.attention(&Request::Claim { id: id.clone() }).unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    store.recover_attention().unwrap();
    assert_eq!(poll(&store)["items"][0]["delivery"], "uncertain");
    assert!(store.attention(&Request::Claim { id }).unwrap().is_null());
    store.lock().unwrap().execute_batch("DROP TABLE attention_notices;DROP TABLE attention_sources;DROP TABLE attention_preferences;PRAGMA user_version=47;").unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert!(poll(&store)["items"].as_array().unwrap().is_empty());
    assert_eq!(
        store.runs("task").unwrap()[0].state,
        dolores_core::RunState::Completed
    );
    assert_eq!(
        store
            .messages_page("task", None, false, 80)
            .unwrap()
            .items
            .len(),
        0
    );
}

#[test]
fn stale_preferences_and_disabled_kind_suppress_pending_without_losing_in_app_notice() {
    let s = s();
    enable(&s);
    begin(&s, RUN);
    s.append_run_event(
        RUN,
        1,
        Some(dolores_core::RunState::WaitingForApproval),
        "approvalRequested",
        &json!({"target":"PRIVATE"}),
    )
    .unwrap();
    let p = poll(&s);
    assert_eq!(p["pendingDelivery"].as_array().unwrap().len(), 1);
    let id = p["items"][0]["id"].as_str().unwrap().to_owned();
    let prefs: Preferences = serde_json::from_value(p["preferences"].clone()).unwrap();
    s.attention(&Request::SavePreferences {
        preferences: Preferences {
            approval: false,
            ..prefs.clone()
        },
    })
    .unwrap();
    assert!(s
        .attention(&Request::SavePreferences { preferences: prefs })
        .unwrap_err()
        .contains("changed"));
    assert!(s.attention(&Request::Claim { id }).unwrap().is_null());
    let p = poll(&s);
    assert_eq!(p["items"][0]["delivery"], "suppressed");
    assert_eq!(p["items"][0]["seen"], false);
    assert!(!p.to_string().contains("PRIVATE"));
}

#[test]
fn an_answered_question_is_suppressed_before_delayed_native_handoff() {
    let s = s();
    enable(&s);
    begin(&s, RUN);
    s.append_run_event(
        RUN,
        1,
        Some(dolores_core::RunState::WaitingForApproval),
        "questionAsked",
        &json!({}),
    )
    .unwrap();
    let page = poll(&s);
    let id = page["items"][0]["id"].as_str().unwrap().to_owned();
    s.append_run_event(
        RUN,
        2,
        Some(dolores_core::RunState::Running),
        "questionAnswered",
        &json!({}),
    )
    .unwrap();
    assert!(s
        .attention(&Request::Claim { id: id.clone() })
        .unwrap()
        .is_null());
    assert_eq!(
        s.attention(&Request::Inspect { id }).unwrap()["delivery"],
        "suppressed"
    );
    assert_eq!(
        s.runs("task").unwrap()[0].state,
        dolores_core::RunState::Running
    );
}
#[test]
fn default_opt_out_exact_dedup_and_stale_decisions_never_change_authority() {
    let s = s();
    begin(&s, RUN);
    s.append_run_event(
        RUN,
        1,
        Some(dolores_core::RunState::WaitingForApproval),
        "questionAsked",
        &json!({"callId":"private-id","questions":"PRIVATE QUESTION"}),
    )
    .unwrap();
    let first = poll(&s);
    let n = &first["items"][0];
    assert_eq!(n["delivery"], "suppressed");
    let id = n["id"].as_str().unwrap().to_owned();
    assert_eq!(poll(&s)["items"].as_array().unwrap().len(), 1);
    assert!(!first.to_string().contains("PRIVATE"));
    enable(&s);
    assert!(s
        .attention(&Request::Claim { id: id.clone() })
        .unwrap()
        .is_null());
    assert_eq!(
        s.attention(&Request::Resolve { id: id.clone() }).unwrap()["sameDecision"],
        true
    );
    s.append_run_event(
        RUN,
        2,
        Some(dolores_core::RunState::Running),
        "questionAnswered",
        &json!({"answer":"private"}),
    )
    .unwrap();
    assert_eq!(
        s.attention(&Request::Resolve { id }).unwrap()["sameDecision"],
        false
    );
    s.append_run_event(
        RUN,
        3,
        Some(dolores_core::RunState::Completed),
        "finished",
        &json!({"response":"private answer"}),
    )
    .unwrap();
    let done = poll(&s);
    assert_eq!(done["items"][0]["kind"], "completed");
    assert_eq!(done["items"][0]["delivery"], "pending");
    assert!(!done.to_string().contains("private"));
    assert_eq!(s.saved_draft("task").unwrap(), "Keep private draft");
    assert_eq!(
        s.runs("task").unwrap()[0].state,
        dolores_core::RunState::Completed
    );
}
#[test]
fn claim_failed_handoff_restart_and_disabled_policy_do_not_redeliver() {
    let s = s();
    enable(&s);
    begin(&s, RUN);
    s.append_run_event(
        RUN,
        1,
        Some(dolores_core::RunState::Failed),
        "finished",
        &json!({"error":"private path"}),
    )
    .unwrap();
    let p = poll(&s);
    let id = p["items"][0]["id"].as_str().unwrap().to_owned();
    assert!(!s
        .attention(&Request::Claim { id: id.clone() })
        .unwrap()
        .is_null());
    assert!(s
        .attention(&Request::Claim { id: id.clone() })
        .unwrap()
        .is_null());
    s.recover_attention().unwrap();
    assert_eq!(poll(&s)["items"][0]["delivery"], "uncertain");
    assert!(s
        .attention(&Request::Claim { id: id.clone() })
        .unwrap()
        .is_null());
    s.attention(&Request::Seen { id: id.clone() }).unwrap();
    assert_eq!(poll(&s)["items"][0]["seen"], true);
    assert!(s
        .attention(&Request::Finish {
            id,
            delivery: Delivery::Accepted
        })
        .is_err());
    assert_eq!(
        s.runs("task").unwrap()[0].state,
        dolores_core::RunState::Failed
    );
}
#[test]
fn failed_attention_save_rolls_back_cursor_without_failing_completed_run() {
    let s = s();
    enable(&s);
    begin(&s, RUN);
    s.append_run_event(
        RUN,
        1,
        Some(dolores_core::RunState::Completed),
        "finished",
        &json!({}),
    )
    .unwrap();
    s.lock().unwrap().execute_batch("CREATE TRIGGER fail_attention BEFORE INSERT ON attention_notices BEGIN SELECT RAISE(ABORT,'fixture unavailable'); END").unwrap();
    assert!(s.attention(&Request::Poll { cursor: None }).is_err());
    assert_eq!(
        s.runs("task").unwrap()[0].state,
        dolores_core::RunState::Completed
    );
    s.lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_attention")
        .unwrap();
    let p = poll(&s);
    assert_eq!(p["items"].as_array().unwrap().len(), 1);
    let id = p["items"][0]["id"].as_str().unwrap().to_owned();
    s.attention(&Request::Claim { id: id.clone() }).unwrap();
    s.attention(&Request::Finish {
        id: id.clone(),
        delivery: Delivery::Failed,
    })
    .unwrap();
    s.attention(&Request::Finish {
        id: id.clone(),
        delivery: Delivery::Failed,
    })
    .unwrap();
    assert!(s.attention(&Request::Claim { id }).unwrap().is_null());
}
#[test]
fn bounded_retention_paging_and_deleted_chat_avoid_replay() {
    let s = s();
    for n in 0..270 {
        if n > 0 && n % 90 == 0 {
            // Respect the existing 100-run per-chat limit; retained attention
            // spans different public tasks rather than bypassing that guard.
            let c = s.lock().unwrap();
            c.execute(
                "INSERT INTO sessions(id,title,updated_at) VALUES(?1,'Public fixture',1)",
                [format!("old-{n}")],
            )
            .unwrap();
            c.execute(
                "UPDATE runs SET thread=?1,data=json_set(data,'$.thread',?1) WHERE thread='task'",
                [format!("old-{n}")],
            )
            .unwrap();
        }
        let id = format!("{:08}-1111-4111-8111-111111111111", n);
        begin(&s, &id);
        s.append_run_event(
            &id,
            1,
            Some(dolores_core::RunState::Completed),
            "finished",
            &json!({}),
        )
        .unwrap();
    }
    while poll(&s)["moreSources"] == true {}
    let first = poll(&s);
    assert_eq!(first["items"].as_array().unwrap().len(), 50);
    assert_eq!(first["omitted"], 14);
    let mut count = 50;
    let mut cursor = first["nextCursor"].as_i64();
    while cursor.is_some() {
        let p = s.attention(&Request::Poll { cursor }).unwrap();
        count += p["items"].as_array().unwrap().len();
        cursor = p["nextCursor"].as_i64();
    }
    assert_eq!(count, 256);
    assert_eq!(poll(&s)["omitted"], 14);
    s.delete("task").unwrap();
    let page = poll(&s);
    assert!(page["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|n| n["session"] != "task"));
}
