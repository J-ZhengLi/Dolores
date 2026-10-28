use std::path::Path;
/// Initial context reads root guidance only, independent of repository size.
pub fn read_scoped_workspace_instructions(root: &Path) -> Result<String, String> {
    crate::read_workspace_instructions(root)
}
pub fn read_path_instructions(root: &Path, path: &str) -> Result<Vec<(String, String)>, String> {
    if !super::valid_path(path) {
        return Err("Invalid guidance path.".into());
    }
    let dir = cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|_| "Working folder unavailable.")?;
    let mut result = vec![];
    let parts: Vec<_> = path.split('/').collect();
    for depth in 1..parts.len() {
        let prefix = parts[..depth].join("/");
        let canonical = dir
            .canonicalize(&prefix)
            .map_err(|_| "Guidance ancestor unavailable.")?;
        if canonical.to_string_lossy().replace('\\', "/") != prefix {
            return Err("Guidance aliases are not loaded.".into());
        }
        let here = dir
            .open_dir(&prefix)
            .map_err(|_| "Guidance ancestor unavailable.")?;
        match here.symlink_metadata("AGENTS.md") {
            Ok(_) => result.push((prefix, super::instructions::read_instruction_dir(&here)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Applicable AGENTS.md cannot be inspected.".into()),
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guidance_is_path_scoped_without_repository_directory_or_depth_caps() {
        let root = tempfile::tempdir().unwrap();
        let chosen = root.path().join("project");
        std::fs::create_dir_all(chosen.join("a/b/c/d/e/f")).unwrap();
        std::fs::write(root.path().join("AGENTS.md"), "PRIVATE_PARENT").unwrap();
        std::fs::write(chosen.join("AGENTS.md"), "root rules").unwrap();
        std::fs::write(chosen.join("a/AGENTS.md"), "a rules 世界").unwrap();
        std::fs::write(chosen.join("a/b/c/d/e/AGENTS.md"), "deep rules").unwrap();
        for i in 0..100 {
            std::fs::create_dir(chosen.join(format!("unrelated-{i}"))).unwrap();
        }
        assert_eq!(
            read_scoped_workspace_instructions(&chosen).unwrap(),
            "root rules"
        );
        let rules = read_path_instructions(&chosen, "a/b/c/d/e/f/new.rs").unwrap();
        assert_eq!(
            rules,
            vec![
                ("a".into(), "a rules 世界".into()),
                ("a/b/c/d/e".into(), "deep rules".into())
            ]
        );
        assert!(read_path_instructions(&chosen, "../private").is_err());
        std::fs::write(chosen.join("a/AGENTS.md"), [0]).unwrap();
        assert!(read_scoped_workspace_instructions(&chosen).is_ok());
        assert!(read_path_instructions(&chosen, "a/new.rs").is_err());
    }
}
