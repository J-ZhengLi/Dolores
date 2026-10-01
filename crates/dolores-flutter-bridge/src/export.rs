use dolores_core::{ExportFormat, SessionStore};
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

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_store_sqlite::SqliteStore;
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
