use super::config::{read, Profile, Snapshot};
use dolores_core::{ProjectSkill, SkillScope};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) fn load(snapshot: &mut Snapshot, profile: &Profile, root: Option<&Path>) {
    let home = profile.home.as_ref().unwrap();
    let mut folders = vec![(home.clone(), SkillScope::Global)];
    if profile.project {
        if let Some(root) = root {
            let mut chain = vec![root.to_path_buf()];
            let mut parent = root;
            while !parent.join(".git").exists() {
                let Some(next) = parent.parent() else { break };
                parent = next;
                chain.push(parent.to_path_buf());
            }
            if !parent.join(".git").exists() {
                chain = vec![root.to_path_buf()];
            }
            chain.reverse();
            folders.extend(chain.into_iter().map(|p| (p, SkillScope::Project)));
        }
    }
    let mut seen = BTreeSet::new();
    for (folder, scope) in folders {
        for name in ["AGENTS.override.md", "AGENTS.md"] {
            let path = folder.join(name);
            match read(&path) {
                Ok(Some(text)) if !text.trim().is_empty() => {
                    instruction(
                        snapshot,
                        &path,
                        &text,
                        profile.referenced_instructions,
                        &mut seen,
                    );
                    break;
                }
                Ok(_) => {}
                Err(e) => snapshot.diagnostics.push(e),
            }
        }
        let candidates = if scope == SkillScope::Global {
            vec![folder.join("skills")]
        } else {
            vec![folder.join(".codex/skills"), folder.join(".agents/skills")]
        };
        for path in candidates {
            if path.is_dir() {
                snapshot.skill_collections.push((path, scope));
            }
        }
    }
}
fn instruction(
    snapshot: &mut Snapshot,
    path: &Path,
    text: &str,
    imports: bool,
    seen: &mut BTreeSet<PathBuf>,
) {
    let Ok(identity) = path.canonicalize() else {
        return;
    };
    if !seen.insert(identity) {
        return;
    }
    snapshot.sources.push(path.to_string_lossy().into());
    snapshot
        .instructions
        .push_str(&format!("\nSource: {}\n{}\n", path.display(), text));
    if !imports {
        return;
    }
    for line in text.lines() {
        let Some(reference) = line.trim().strip_prefix('@') else {
            continue;
        };
        let reference = reference.trim().trim_matches('"');
        // Explicit standalone file references are a Dolores extension, not a
        // promise to evaluate arbitrary Markdown or shell expansions.
        if reference.is_empty() || reference.contains(['\n', '\r', '\0']) {
            continue;
        }
        let path = if let Some(rest) = reference
            .strip_prefix("~/")
            .or_else(|| reference.strip_prefix("~\\"))
        {
            directories::BaseDirs::new().map(|p| p.home_dir().join(rest))
        } else {
            let referenced = Path::new(reference);
            Some(if referenced.is_absolute() {
                referenced.into()
            } else {
                path.parent().unwrap().join(referenced)
            })
        };
        if let Some(path) = path {
            match read(&path) {
                Ok(Some(text)) => instruction(snapshot, &path, &text, true, seen),
                Ok(None) => snapshot.diagnostics.push(format!(
                    "Referenced instruction {} is missing.",
                    path.display()
                )),
                Err(e) => snapshot.diagnostics.push(e),
            }
        }
    }
}
pub(crate) fn skills(snapshot: &Snapshot, existing: &mut Vec<ProjectSkill>) {
    for (collection, scope) in &snapshot.skill_collections {
        let Ok(entries) = std::fs::read_dir(collection) else {
            continue;
        };
        let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !dolores_core::valid_skill_name(&name) {
                continue;
            }
            // Individual skill links are supported by Codex. Bind identity to
            // their resolved source, never copy or mutate their targets.
            let source = entry.path().join("SKILL.md");
            let Ok(Some(text)) = read(&source) else {
                continue;
            };
            let Ok(document) = dolores_tools_fs::parse_skill_document(&name, text) else {
                continue;
            };
            let identity = source.canonicalize().unwrap_or(source);
            let mut hash = Sha256::new();
            hash.update(identity.to_string_lossy().as_bytes());
            hash.update([0]);
            hash.update(document.text.as_bytes());
            let digest = hash.finalize();
            let skill = ProjectSkill {
                scope: *scope,
                name: name.clone(),
                enabled: true,
                revision: u32::from_be_bytes(digest[..4].try_into().unwrap()).max(1),
                versions: vec![dolores_core::SkillVersion {
                    version: u32::from_be_bytes(digest[4..8].try_into().unwrap()).max(1),
                    reviewed_at: 0,
                    rollback_from: None,
                    evaluation: None,
                    package: None,
                    document,
                }],
            };
            if let Some(index) = existing
                .iter()
                .position(|s| s.scope == *scope && s.name == name)
            {
                // Preserve explicit opt-outs and managed package ownership.
                if existing[index].enabled
                    && existing[index].current().package.is_none()
                    && existing[index].current().evaluation.is_none()
                    && existing[index].current().rollback_from.is_none()
                {
                    existing[index] = skill;
                }
            } else {
                existing.push(skill);
            }
        }
    }
}
