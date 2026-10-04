use cap_std::fs::{Dir, OpenOptions};
use dolores_core::MAX_INSTRUCTION_BYTES;
use std::{io::Read, path::Path};

/// User-initiated review of one literal root file. Never resolves includes.
pub fn read_workspace_instructions(root: &Path) -> Result<String, String> {
    if !root.is_absolute() {
        return Err("Choose an absolute working folder.".into());
    }
    let dir = Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|_| "Working folder is unavailable.")?;
    read_instruction_dir(&dir)
}
pub(super) fn read_instruction_dir(dir: &Dir) -> Result<String, String> {
    let before = dir
        .symlink_metadata("AGENTS.md")
        .map_err(|_| "AGENTS.md is missing or unreadable in this working folder.")?;
    if !before.is_file() || before.file_type().is_symlink() {
        return Err(
            "AGENTS.md must be a regular file in the working folder; links are not loaded.".into(),
        );
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    let file = dir
        .open_with("AGENTS.md", &options)
        .map_err(|_| "AGENTS.md could not be opened.")?;
    if !file
        .metadata()
        .map_err(|_| "AGENTS.md is unavailable.")?
        .is_file()
        || dir
            .symlink_metadata("AGENTS.md")
            .map_err(|_| "AGENTS.md is unavailable.")?
            .file_type()
            .is_symlink()
    {
        return Err("AGENTS.md must be a regular file; review it again.".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_INSTRUCTION_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "AGENTS.md could not be read.")?;
    if bytes.len() > MAX_INSTRUCTION_BYTES || bytes.contains(&0) {
        return Err(
            "AGENTS.md must be UTF-8 text within 8 KiB; binary files are not loaded.".into(),
        );
    }
    let text = String::from_utf8(bytes).map_err(|_| "AGENTS.md is not UTF-8 text.")?;
    if text.trim().is_empty() {
        return Err("AGENTS.md is empty. Add guidance before enabling it.".into());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn root_only_review_rejects_missing_directories_binary_utf8_and_size_and_keeps_includes_literal(
    ) {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("AGENTS.md");
        std::fs::create_dir(root.path().join("nested")).unwrap();
        std::fs::write(root.path().join("nested/AGENTS.md"), "not root").unwrap();
        assert!(read_workspace_instructions(root.path()).is_err());
        for invalid in [vec![0], vec![255], vec![b' '; 10], vec![b'x'; 8193]] {
            std::fs::write(&path, invalid).unwrap();
            assert!(read_workspace_instructions(root.path()).is_err());
        }
        std::fs::write(&path, "# Guidance\n@../private.env\nUse tests. 世界").unwrap();
        assert!(read_workspace_instructions(root.path())
            .unwrap()
            .contains("@../private.env"));
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(read_workspace_instructions(root.path()).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn linked_guidance_is_refused_even_for_inside_targets() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("guidance"), "hello").unwrap();
        std::os::unix::fs::symlink("guidance", root.path().join("AGENTS.md")).unwrap();
        assert!(read_workspace_instructions(root.path()).is_err());
    }
}
