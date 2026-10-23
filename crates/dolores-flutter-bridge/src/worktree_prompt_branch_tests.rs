use super::*;
use crate::prompt_branches::Request as Branch;
use dolores_core::prompt_branches::{Create, Folder};

#[test]
fn managed_prompt_branch_keeps_source_staging_and_dirty_work_and_refuses_active_or_reused_targets()
{
    let f = Fixture::new();
    std::fs::write(f.root.join("note.txt"), "staged source\n").unwrap();
    git(&f.root, &["add", "note.txt"]);
    std::fs::write(f.root.join("note.txt"), "source dirty work after staging\n").unwrap();
    let index = std::fs::read(f.root.join(".git/index")).unwrap();
    f.store
        .commit_turn(&f.source, "Original prompt", "Original answer")
        .unwrap();
    f.store
        .save_draft(&f.source, "Keep source draft 世界")
        .unwrap();
    let message = f
        .store
        .messages_page(&f.source, None, false, 80)
        .unwrap()
        .items[0]
        .id;
    let basis = f
        .store
        .preview_prompt_branch(&f.source, message)
        .unwrap()
        .basis;
    let record = f.create();
    let call = |request| {
        f.engine.call(Command::PromptBranch {
            session: f.source.clone(),
            request,
        })
    };
    let selected = call(Branch::Target {
        target: record.session.clone(),
        revision: record.revision,
    })
    .unwrap();
    assert_eq!(selected["baseCommit"], record.base_commit);
    let request = Create {
        id: record.session.clone(),
        basis,
        edited: "Inspect this managed checkout".into(),
        include_attachments: false,
        include_context: false,
        folder: Folder::Managed,
        target_revision: Some(record.revision),
    };
    f.engine
        .active
        .lock()
        .unwrap()
        .reserve(crate::Run {
            thread: Some(record.session.clone()),
            id: 910,
            cancel: CancellationToken::new(),
            events: tokio::sync::mpsc::channel(1).1,
            approvals: Arc::new(Mutex::new(None)),
        })
        .unwrap();
    assert!(call(Branch::Create {
        request: request.clone()
    })
    .unwrap_err()
    .contains("active task"));
    assert!(f.store.prompt_branch(&record.session).unwrap().is_none());
    f.engine.active.lock().unwrap().remove(910);
    let made = call(Branch::Create {
        request: request.clone(),
    })
    .unwrap();
    assert_eq!(made["session"]["id"], record.session);
    assert_eq!(
        made["info"]["origin"]["managed"]["baseCommit"],
        record.base_commit
    );
    assert_eq!(
        std::fs::read_to_string(Path::new(&record.root).join("note.txt")).unwrap(),
        "base\n"
    );
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "source dirty work after staging\n"
    );
    assert_eq!(std::fs::read(f.root.join(".git/index")).unwrap(), index);
    assert_eq!(
        f.store.saved_draft(&f.source).unwrap(),
        "Keep source draft 世界"
    );
    assert_eq!(
        f.store.saved_draft(&record.session).unwrap(),
        "Inspect this managed checkout"
    );
    assert!(f.store.runs(&record.session).unwrap().is_empty());
    f.store
        .save_draft(&record.session, "Newer managed draft")
        .unwrap();
    call(Branch::Create { request }).unwrap();
    assert_eq!(
        f.store.saved_draft(&record.session).unwrap(),
        "Newer managed draft"
    );
    assert!(call(Branch::Target {
        target: record.session.clone(),
        revision: record.revision
    })
    .unwrap_err()
    .contains("already has work"));
}
