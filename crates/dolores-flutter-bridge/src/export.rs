use dolores_core::{ExportFormat, SessionStore, SkillDocument};
use std::{
    io::{BufWriter, Write},
    path::Path,
};

pub(super) fn save(
    store: &dyn SessionStore,
    session: &str,
    path: &Path,
    format: ExportFormat,
) -> Result<u64, String> {
    let extension = match format {
        ExportFormat::Markdown => "md",
        ExportFormat::Json => "json",
    };
    if !path.is_absolute()
        || path
            .extension()
            .and_then(|s| s.to_str())
            .is_none_or(|s| !s.eq_ignore_ascii_case(extension))
    {
        return Err(format!(
            "Choose an absolute export path ending in .{extension}."
        ));
    }
    // Never replace existing files (including application data). Publication happens
    // only after a complete snapshot is written; errors drop the temporary file.
    if path.exists() {
        return Err("That file already exists. Choose a new export filename.".into());
    }
    let parent = path.parent().ok_or("Choose an export folder.")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|_| "Could not create the export file.")?;
    let count = {
        let mut writer = BufWriter::new(temporary.as_file_mut());
        let count = store.export_conversation(session, format, &mut writer)?;
        writer
            .flush()
            .map_err(|_| "Could not finish the export file.")?;
        count
    };
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| "Could not finish the export file.")?;
    temporary
        .persist_noclobber(path)
        .map_err(|_| "Could not save the export. Choose a new filename in a writable folder.")?;
    Ok(count)
}

pub(super) fn save_skill(document: &SkillDocument, path: &Path) -> Result<usize, String> {
    document.validate()?;
    // Preserve the entire retained document, including optional standard metadata.
    // Reject inconsistent legacy records rather than silently rewriting the snapshot.
    if dolores_tools_fs::parse_skill_document(&document.name, document.text.clone())? != *document {
        return Err(
            "Saved skill metadata does not match its document. Review another version.".into(),
        );
    }
    let parent = path.parent().ok_or("Choose a skill export folder.")?;
    if !path.is_absolute()
        || path.file_name().and_then(|n| n.to_str()) != Some("SKILL.md")
        || parent.file_name().and_then(|n| n.to_str()) != Some(document.name.as_str())
    {
        return Err(format!(
            "Save as SKILL.md inside a folder named {}.",
            document.name
        ));
    }
    if path.symlink_metadata().is_ok() {
        return Err(
            "That file already exists. Choose a new folder; existing files are never overwritten."
                .into(),
        );
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|_| "Choose an existing writable skill folder.")?;
    temporary
        .write_all(document.text.as_bytes())
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|_| "Could not finish the skill export.")?;
    // Atomic no-clobber publication also refuses a destination created after the check.
    temporary.persist_noclobber(path).map_err(|_| {
        "Could not save the skill. Choose a new folder; existing files are never overwritten."
    })?;
    Ok(document.text.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_store_sqlite::SqliteStore;
    use std::path::PathBuf;
    #[test]
    fn skill_export_preserves_standard_document_and_refuses_invalid_or_existing_targets() {
        let directory = tempfile::tempdir().unwrap();
        let folder = directory.path().join("review");
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("SKILL.md");
        let text = "---\r\nname: review\r\ndescription: >-\r\n  Review code\r\n  and tests.\r\nmetadata:\r\n  version: '1'\r\n---\r\nCheck tests. 世界\r\n";
        let document = dolores_tools_fs::parse_skill_document("review", text.into()).unwrap();
        assert_eq!(save_skill(&document, &path).unwrap(), text.len());
        assert_eq!(std::fs::read(&path).unwrap(), text.as_bytes());
        assert_eq!(
            dolores_tools_fs::read_global_skill(directory.path(), "review").unwrap(),
            document
        );
        assert!(save_skill(&document, &path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), text.as_bytes());
        std::fs::write(&path, "").unwrap();
        assert!(save_skill(&document, &path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"");
        for target in [
            PathBuf::from("review/SKILL.md"),
            folder.join("wrong.md"),
            directory.path().join("SKILL.md"),
        ] {
            assert!(save_skill(&document, &target).is_err());
        }
        std::fs::remove_file(&path).unwrap();
        let mut invalid = document.clone();
        invalid.description = "Different metadata".into();
        assert!(save_skill(&invalid, &path).is_err());
        invalid.text = "No frontmatter".into();
        assert!(save_skill(&invalid, &path).is_err());
        assert!(!path.exists());
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
        std::fs::create_dir(&path).unwrap();
        assert!(save_skill(&document, &path).is_err());
        assert!(path.is_dir());
    }
    #[test]
    fn concurrent_skill_exports_publish_once_without_partial_or_temporary_files() {
        use std::{
            path::PathBuf,
            sync::{Arc, Barrier},
        };
        let directory = tempfile::tempdir().unwrap();
        let folder = directory.path().join("review");
        std::fs::create_dir(&folder).unwrap();
        let path: PathBuf = folder.join("SKILL.md");
        let document =
            dolores_core::skill_document("review", "Review code", "Check tests. 世界").unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let threads: Vec<_> = (0..2)
            .map(|_| {
                let (path, document, barrier) = (path.clone(), document.clone(), barrier.clone());
                std::thread::spawn(move || {
                    barrier.wait();
                    save_skill(&document, &path)
                })
            })
            .collect();
        let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(std::fs::read(&path).unwrap(), document.text.as_bytes());
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[test]
    fn skill_export_refuses_dangling_destination_links() {
        let directory = tempfile::tempdir().unwrap();
        let folder = directory.path().join("review");
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("SKILL.md");
        let missing = folder.join("missing");
        std::os::unix::fs::symlink(&missing, &path).unwrap();
        let document = dolores_core::skill_document("review", "Review", "Check tests").unwrap();
        assert!(save_skill(&document, &path).is_err());
        assert!(path.symlink_metadata().unwrap().file_type().is_symlink());
        assert!(!missing.exists());
    }
    #[test]
    fn export_publishes_complete_file_and_never_replaces_or_leaves_partial_files() {
        let directory = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&directory.path().join("data.db")).unwrap();
        store.create("test").unwrap();
        store.commit_turn("test", "你好", "complete").unwrap();
        let path = directory.path().join("chat.json");
        assert_eq!(save(&store, "test", &path, ExportFormat::Json).unwrap(), 2);
        let original = std::fs::read(&path).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        assert_eq!(value["messages"][0]["content"], "你好");
        assert!(save(&store, "test", &path, ExportFormat::Json).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
        let failed = directory.path().join("missing.json");
        assert!(save(&store, "deleted", &failed, ExportFormat::Json).is_err());
        assert!(!failed.exists());
        let corrupted = rusqlite::Connection::open(directory.path().join("data.db")).unwrap();
        corrupted
            .execute(
                "UPDATE messages SET role='invalid' WHERE role='assistant'",
                [],
            )
            .unwrap();
        // A failure after the header and first message were written must not
        // publish a partial destination or leave its temporary file behind.
        assert!(save(&store, "test", &failed, ExportFormat::Json).is_err());
        assert!(!failed.exists());
        assert!(save(
            &store,
            "test",
            Path::new("relative.json"),
            ExportFormat::Json
        )
        .is_err());
        assert!(save(
            &store,
            "test",
            &directory.path().join("data.db"),
            ExportFormat::Json
        )
        .is_err());
        assert!(!std::fs::read_dir(directory.path())
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".tmp")));
    }
}
