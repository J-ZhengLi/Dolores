use std::path::Path;
pub fn read_scoped_workspace_instructions(root: &Path) -> Result<String, String> {
    let root_text = crate::read_workspace_instructions(root)?;
    let dir = cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|_| "Working folder is unavailable.")?;
    let mut files = vec![];
    let mut queue = vec![(String::new(), 0)];
    let mut seen = 0;
    while let Some((prefix, depth)) = queue.pop() {
        let here = if prefix.is_empty() {
            dir.try_clone()
        } else {
            dir.open_dir(&prefix)
        }
        .map_err(|_| "Guidance directory could not be inspected.")?;
        let mut entries = here
            .entries()
            .map_err(|_| "Guidance directory could not be inspected.")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Guidance directory could not be inspected.")?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let meta = entry
                .metadata()
                .map_err(|_| "Guidance entry could not be inspected.")?;
            if name.starts_with('.')
                || ["target", "build", "node_modules", "output"].contains(&name.as_str())
            {
                continue;
            }
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            if entry
                .file_type()
                .map_err(|_| "Guidance entry is unavailable.")?
                .is_symlink()
            {
                continue;
            }
            if meta.is_dir() && depth < 4 {
                seen += 1;
                if seen > 64 {
                    return Err(
                        "Guidance scan exceeds 64 directories. Choose a narrower working folder."
                            .into(),
                    );
                }
                queue.push((path, depth + 1));
            }
            if name == "AGENTS.md" && !prefix.is_empty() {
                if files.len() >= 8 {
                    return Err(
                        "Guidance exceeds eight nested files. Choose a narrower folder.".into(),
                    );
                }
                let text = super::instructions::read_instruction_dir(&here)?;
                files.push((prefix.clone(), text));
            }
        }
    }
    if files.is_empty() {
        return Ok(root_text);
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut text = format!(
        "Scope: ./ (root AGENTS.md; applies throughout chosen working folder)\n{root_text}\n"
    );
    for (prefix, body) in files {
        text.push_str(&format!("\nScope: {prefix}/ (nested AGENTS.md; applies only below this path; deeper guidance wins for that path)\n{body}\n"));
    }
    if text.len() > dolores_core::MAX_INSTRUCTION_BYTES {
        return Err("Scoped guidance exceeds 8 KiB. Narrow the working folder or shorten guidance; nothing was activated.".into());
    }
    Ok(text)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_scope_is_bounded_and_parent_is_not_loaded() {
        let root = tempfile::tempdir().unwrap();
        let chosen = root.path().join("project");
        std::fs::create_dir_all(chosen.join("src/nested")).unwrap();
        std::fs::write(root.path().join("AGENTS.md"), "PRIVATE_PARENT").unwrap();
        std::fs::write(chosen.join("AGENTS.md"), "root rules").unwrap();
        std::fs::write(chosen.join("src/AGENTS.md"), "src rules 世界").unwrap();
        let text = read_scoped_workspace_instructions(&chosen).unwrap();
        assert!(text.contains("Scope: src/"));
        assert!(!text.contains("PRIVATE_PARENT"));
        std::fs::write(chosen.join("src/AGENTS.md"), "x".repeat(8192)).unwrap();
        assert!(read_scoped_workspace_instructions(&chosen).is_err());
    }
}
