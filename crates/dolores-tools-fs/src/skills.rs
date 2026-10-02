use cap_std::fs::{Dir, OpenOptions};
use dolores_core::{valid_skill_name, SkillDocument, MAX_SKILL_BYTES};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};

#[derive(Serialize)]
pub struct SkillCatalog {
    pub names: Vec<String>,
    pub sources: Vec<String>,
    pub partial: bool,
}
fn skills_dir(root: &Path) -> Result<Dir, String> {
    if !root.is_absolute() {
        return Err("Choose an absolute working folder.".into());
    }
    let mut dir = Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|_| "Working folder is unavailable.")?;
    for part in [".agents", "skills"] {
        let info = dir
            .symlink_metadata(part)
            .map_err(|_| "No readable .agents/skills directory in this working folder.")?;
        if !info.is_dir()
            || info.file_type().is_symlink()
            || dir.canonicalize(part).ok().as_deref() != Some(Path::new(part))
        {
            return Err(
                "Project skill directories must be direct directories; links are not loaded."
                    .into(),
            );
        }
        dir = dir
            .open_dir(part)
            .map_err(|_| "Project skill directory is unavailable.")?;
    }
    Ok(dir)
}
/// Only the explicitly configured global collection boundary may be a link.
pub fn global_skills_target(root: &Path) -> Result<std::path::PathBuf, String> {
    if !root.is_absolute() {
        return Err("Global skills need an absolute directory.".into());
    }
    let target = std::fs::canonicalize(root).map_err(|_| {
        "Global skills directory is unavailable. Add ~/.agents/skills, then Refresh."
    })?;
    if !target.is_dir() {
        return Err("Global skills need a directory.".into());
    }
    Ok(target)
}
fn open_collection(root: &Path, global: bool) -> Result<Dir, String> {
    if !global {
        return skills_dir(root);
    }
    Dir::open_ambient_dir(global_skills_target(root)?, cap_std::ambient_authority())
        .map_err(|_| "Global skills directory is unavailable.".into())
}
pub fn project_skill_catalog(root: &Path) -> Result<SkillCatalog, String> {
    catalog(open_collection(root, false)?)
}
pub fn global_skill_catalog(root: &Path) -> Result<SkillCatalog, String> {
    catalog(open_collection(root, true)?)
}
fn catalog(dir: Dir) -> Result<SkillCatalog, String> {
    let entries = dir
        .entries()
        .map_err(|_| "Skill names could not be read.")?;
    let mut names = vec![];
    let mut partial = false;
    for (index, entry) in entries.enumerate() {
        if index == 128 {
            partial = true;
            break;
        }
        let entry = entry.map_err(|_| "Skill names could not be read.")?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !valid_skill_name(name) || !super::valid_path(&format!(".agents/skills/{name}/SKILL.md"))
        {
            continue;
        }
        let info = dir
            .symlink_metadata(name)
            .map_err(|_| "Skill names changed; refresh Skills.")?;
        if info.is_dir()
            && !info.file_type().is_symlink()
            && dir.canonicalize(name).ok().as_deref() == Some(Path::new(name))
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    if names.len() > 32 {
        partial = true;
        names.truncate(32);
    }
    let sources = names
        .iter()
        .filter(|name| {
            let path = format!("{name}/SKILL.md");
            dir.symlink_metadata(&path)
                .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
                && dir.canonicalize(&path).ok().as_deref() == Some(Path::new(&path))
        })
        .cloned()
        .collect();
    Ok(SkillCatalog {
        names,
        sources,
        partial,
    })
}

#[derive(Deserialize)]
struct Metadata {
    name: String,
    description: String,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    compatibility: Option<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
    #[serde(default, rename = "allowed-tools")]
    allowed_tools: Option<String>,
}
fn parse(name: &str, text: String) -> Result<SkillDocument, String> {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return Err("SKILL.md needs YAML frontmatter with name and description.".into());
    }
    let mut header = vec![];
    let mut closed = false;
    for line in lines.by_ref() {
        if line == "---" {
            closed = true;
            break;
        }
        header.push(line);
    }
    if !closed || lines.all(|l| l.trim().is_empty()) {
        return Err("SKILL.md needs closed YAML frontmatter and nonempty instructions.".into());
    }
    let data: Metadata = serde_yaml_ng::from_str(&header.join("\n"))
        .map_err(|_| "SKILL.md metadata is invalid. Use YAML name and description strings.")?;
    if data.name != name
        || data
            .compatibility
            .as_ref()
            .is_some_and(|v| v.trim().is_empty() || v.chars().count() > 500)
    {
        return Err(
            "SKILL.md name must match its directory; compatibility must be within 500 characters."
                .into(),
        );
    }
    // Parse optional standard metadata, but grant no capabilities from it.
    let _ = (data.license, data.metadata, data.allowed_tools);
    let document = SkillDocument {
        name: data.name,
        description: data.description,
        text,
    };
    document.validate()?;
    Ok(document)
}
pub fn read_project_skill(root: &Path, name: &str) -> Result<SkillDocument, String> {
    read_skill(root, name, false)
}
pub fn read_global_skill(root: &Path, name: &str) -> Result<SkillDocument, String> {
    read_skill(root, name, true)
}
fn read_skill(root: &Path, name: &str, global: bool) -> Result<SkillDocument, String> {
    if !valid_skill_name(name) || !super::valid_path(&format!(".agents/skills/{name}/SKILL.md")) {
        return Err("Skill name is invalid.".into());
    }
    let target = if global {
        Some(global_skills_target(root)?)
    } else {
        None
    };
    let dir = open_collection(root, global)?;
    let info = dir
        .symlink_metadata(name)
        .map_err(|_| "Skill directory is missing.")?;
    if !info.is_dir()
        || info.file_type().is_symlink()
        || dir.canonicalize(name).ok().as_deref() != Some(Path::new(name))
    {
        return Err("Individual skill directories cannot be links.".into());
    }
    let dir = dir
        .open_dir(name)
        .map_err(|_| "Skill directory is unavailable.")?;
    let info = dir
        .symlink_metadata("SKILL.md")
        .map_err(|_| "SKILL.md is missing or unreadable.")?;
    if !info.is_file()
        || info.file_type().is_symlink()
        || dir.canonicalize("SKILL.md").ok().as_deref() != Some(Path::new("SKILL.md"))
    {
        return Err("SKILL.md must be a regular file; links are not loaded.".into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    let file = dir
        .open_with("SKILL.md", &options)
        .map_err(|_| "SKILL.md could not be opened.")?;
    if !file
        .metadata()
        .map_err(|_| "SKILL.md is unavailable.")?
        .is_file()
    {
        return Err("SKILL.md must be a regular file.".into());
    }
    let mut bytes = vec![];
    file.take((MAX_SKILL_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "SKILL.md could not be read.")?;
    if bytes.len() > MAX_SKILL_BYTES || bytes.contains(&0) {
        return Err("SKILL.md must be UTF-8 text within 8 KiB, without NUL.".into());
    }
    let text = String::from_utf8(bytes).map_err(|_| "SKILL.md is not UTF-8 text.")?;
    let document = parse(name, text)?;
    // Re-resolve from the root to detect replacement/link changes during reading.
    let current = open_collection(root, global)?;
    if (global && Some(global_skills_target(root)?) != target)
        || current.canonicalize(name).ok().as_deref() != Some(Path::new(name))
        || dir
            .symlink_metadata("SKILL.md")
            .map_err(|_| "SKILL.md changed; review it again.")?
            .file_type()
            .is_symlink()
    {
        return Err("Skill path changed; review it again.".into());
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_yaml_exact_text_bounds_and_root_only_discovery() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        assert!(project_skill_catalog(root).is_err());
        std::fs::create_dir_all(root.join(".agents/skills/review")).unwrap();
        let file = root.join(".agents/skills/review/SKILL.md");
        let text = "---\r\nname: review\r\ndescription: >-\r\n  Review code\r\n  and tests.\r\nmetadata:\r\n  version: '1'\r\nallowed-tools: everything\r\n---\r\nUse scripts/local.py. @../private.env 世界";
        std::fs::write(&file, text).unwrap();
        let doc = read_project_skill(root, "review").unwrap();
        assert_eq!(doc.text, text);
        assert_eq!(doc.description, "Review code and tests.");
        assert_eq!(project_skill_catalog(root).unwrap().names, vec!["review"]);
        for invalid in [
            "---\nname: wrong\ndescription: desc\n---\nBody",
            "---\nname: review\nname: review\ndescription: desc\n---\nBody",
            "---\nname: review\ndescription: desc\n---\n",
            "name: review",
            "\0",
            &"x".repeat(8193),
        ] {
            std::fs::write(&file, invalid).unwrap();
            assert!(read_project_skill(root, "review").is_err());
        }
        assert!(read_project_skill(root, "../review").is_err());
        for i in 0..40 {
            std::fs::create_dir(root.join(format!(".agents/skills/skill-{i}"))).unwrap();
        }
        let catalog = project_skill_catalog(root).unwrap();
        assert_eq!(catalog.names.len(), 32);
        assert!(catalog.partial);
    }
    #[cfg(unix)]
    #[test]
    fn symlinked_skill_directories_and_files_are_refused() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join(".agents/skills/review")).unwrap();
        std::fs::write(root.join("other"), "text").unwrap();
        symlink(
            root.join("other"),
            root.join(".agents/skills/review/SKILL.md"),
        )
        .unwrap();
        assert!(read_project_skill(root, "review").is_err());
        symlink("review", root.join(".agents/skills/linked")).unwrap();
        assert!(!project_skill_catalog(root)
            .unwrap()
            .names
            .contains(&"linked".into()));
        let boundary = root.join("global-skills");
        symlink(root.join(".agents/skills"), &boundary).unwrap();
        assert!(!global_skill_catalog(&boundary)
            .unwrap()
            .names
            .contains(&"linked".into()));
        assert!(read_global_skill(&boundary, "review").is_err());
        std::fs::remove_file(root.join(".agents/skills/review/SKILL.md")).unwrap();
        std::fs::write(
            root.join(".agents/skills/review/SKILL.md"),
            "---\nname: review\ndescription: Review\n---\nGlobal body",
        )
        .unwrap();
        assert_eq!(
            read_global_skill(&boundary, "review").unwrap().description,
            "Review"
        );
    }
    #[cfg(windows)]
    #[test]
    fn junctions_cannot_redirect_any_skill_directory_component() {
        use std::os::windows::process::CommandExt;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join(".agents/skills/review")).unwrap();
        std::fs::write(
            root.join(".agents/skills/review/SKILL.md"),
            "---\nname: review\ndescription: Review\n---\nBody",
        )
        .unwrap();
        let link = |from: &Path, to: &Path| {
            let result = std::process::Command::new("cmd")
                .creation_flags(0x08000000)
                .args(["/c", "mklink", "/J"])
                .arg(from)
                .arg(to)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "Could not create isolated test junction"
            );
        };
        let agents = root.join(".agents");
        let skills = agents.join("skills");
        link(&skills.join("linked"), &skills.join("review"));
        assert!(!project_skill_catalog(root)
            .unwrap()
            .names
            .contains(&"linked".into()));
        assert!(read_project_skill(root, "linked").is_err());
        let other = tempfile::tempdir().unwrap();
        link(&other.path().join(".agents"), &agents);
        assert!(project_skill_catalog(other.path()).is_err());
        let nested = tempfile::tempdir().unwrap();
        std::fs::create_dir(nested.path().join(".agents")).unwrap();
        link(&nested.path().join(".agents").join("skills"), &skills);
        assert!(read_project_skill(nested.path(), "review").is_err());
        let global = other.path().join("global-skills");
        link(&global, &skills);
        assert_eq!(
            read_global_skill(&global, "review").unwrap().description,
            "Review"
        );
        assert!(!global_skill_catalog(&global)
            .unwrap()
            .names
            .contains(&"linked".into()));
        assert!(read_global_skill(&global, "linked").is_err());
        assert_eq!(
            global_skills_target(&global).unwrap(),
            std::fs::canonicalize(&skills).unwrap()
        );
    }
}
