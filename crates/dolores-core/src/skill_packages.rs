//! Portable package admission; metadata declares requirements, never authority.
use serde::{Deserialize, Serialize};

pub const MAX_ARCHIVE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PACKAGE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_RESOURCE_BYTES: usize = 512 * 1024;
pub const MAX_PACKAGE_FILES: usize = 64;
pub const MAX_PACKAGE_ENTRIES: usize = 128;
pub const MAX_MANIFEST_BYTES: usize = 8192;
pub const MAX_INSTALLED_BUNDLES: usize = 60;
pub const GLOBAL_ROOT: &str = "@global-skills";

pub fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    pub id: String,
    pub content_digest: String,
    pub manifest: Manifest,
    pub resources: Vec<Resource>,
    pub directories: Vec<String>,
    pub root: String,
}
impl Snapshot {
    pub fn validate(&self) -> Result<(), String> {
        self.manifest.validate()?;
        if !valid_digest(&self.id)
            || !valid_digest(&self.content_digest)
            || !std::path::Path::new(&self.root).is_absolute()
            || self.root.len() > 4096
            || self.root.chars().any(char::is_control)
            || self.resources.is_empty()
            || self.resources.len() > MAX_PACKAGE_FILES
            || self.directories.len() > MAX_PACKAGE_ENTRIES * 12
        {
            return Err(
                "Saved skill package is invalid. Review or reinstall the retained ZIP.".into(),
            );
        }
        let mut names = std::collections::BTreeMap::new();
        let mut total = 0;
        for resource in &self.resources {
            validate_path(&resource.path)?;
            let limit = match resource.path.as_str() {
                "SKILL.md" => crate::MAX_SKILL_BYTES,
                "dolores-package.json" => MAX_MANIFEST_BYTES,
                _ => MAX_RESOURCE_BYTES,
            };
            if resource.bytes > limit
                || !valid_digest(&resource.digest)
                || names.insert(resource.path.to_lowercase(), false).is_some()
            {
                return Err("Saved skill package resources are invalid. Reinstall or choose another version.".into());
            }
            total += resource.bytes;
        }
        for path in &self.directories {
            validate_path(path)?;
            if names.insert(path.to_lowercase(), true).is_some() {
                return Err(
                    "Saved skill package directories collide. Reinstall the package.".into(),
                );
            }
        }
        for (path, directory) in &names {
            if !directory
                && names
                    .keys()
                    .any(|other| other.starts_with(&format!("{path}/")))
            {
                return Err("Saved package has a file/directory collision.".into());
            }
        }
        let directories = self
            .directories
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        for path in self
            .resources
            .iter()
            .map(|r| &r.path)
            .chain(self.directories.iter())
        {
            let mut parent = path.rsplit_once('/').map(|(p, _)| p);
            while let Some(p) = parent {
                if !directories.contains(&p.to_owned()) {
                    return Err("Package parent directories are missing or differ in case.".into());
                }
                parent = p.rsplit_once('/').map(|(p, _)| p);
            }
        }
        if total > MAX_PACKAGE_BYTES || !self.resources.iter().any(|r| r.path == "SKILL.md") {
            return Err("Saved package exceeds its allowance or has no SKILL.md.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Installed {
    pub document: crate::SkillDocument,
    pub package: Snapshot,
    pub installed_at: i64,
}
impl Installed {
    pub fn validate(&self) -> Result<(), String> {
        self.document.validate()?;
        self.package.validate()?;
        if self.installed_at < 0
            || self
                .package
                .resources
                .iter()
                .find(|r| r.path == "SKILL.md")
                .is_none_or(|r| r.bytes != self.document.text.len())
        {
            return Err("Saved package document is inconsistent. Reinstall its ZIP.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dependency {
    pub name: String,
    pub requirement: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}
impl Default for Manifest {
    fn default() -> Self {
        Self {
            schema: 1,
            version: "unversioned".into(),
            origin: None,
            dependencies: vec![],
        }
    }
}
impl Manifest {
    pub fn validate(&self) -> Result<(), String> {
        let text = |s: &str, max: usize| {
            !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
        };
        if self.schema != 1
            || !text(&self.version, 64)
            || self.origin.as_ref().is_some_and(|s| !text(s, 512))
            || self.dependencies.len() > 16
            || self
                .dependencies
                .iter()
                .any(|d| !text(&d.name, 128) || !text(&d.requirement, 256))
        {
            return Err("Package metadata is invalid. Use schema 1, a version within 64 bytes, origin within 512 bytes and at most 16 named dependencies.".into());
        }
        let names: std::collections::BTreeSet<_> = self
            .dependencies
            .iter()
            .map(|d| d.name.to_lowercase())
            .collect();
        if names.len() != self.dependencies.len() {
            return Err("Package dependencies contain duplicate names. Inspect the manifest before importing.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub path: String,
    pub bytes: usize,
    pub digest: String,
}

/// Use identical rules on every host so an archive reviewed elsewhere stays safe
/// to publish on Windows. Reject aliases instead of normalizing their targets.
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.len() > 240 || path.contains('\\') || path.split('/').count() > 12 {
        return Err("Package paths must be relative forward-slash paths within 240 bytes and 12 components.".into());
    }
    for part in path.split('/') {
        let base = part
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>:\"|?*".contains(c))
            || matches!(
                base.as_str(),
                "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
            )
            || ["COM", "LPT"].iter().any(|prefix| {
                base.strip_prefix(prefix).is_some_and(|s| {
                    matches!(
                        s,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
            })
        {
            return Err("Package path escapes or aliases a Windows filename. Remove rooted paths, links, device names and alternate streams before importing.".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_paths_reject_windows_aliases_and_keep_unicode_resources() {
        for invalid in [
            "",
            "/root",
            "C:/root",
            "//server/share",
            "a\\b",
            "../out",
            "a/./b",
            "a//b",
            "a/../b",
            "NUL.txt",
            "con",
            "scripts/COM1.py",
            "LPT².log",
            "a:b",
            "a.",
            "a ",
            "a/\0b",
        ] {
            assert!(validate_path(invalid).is_err(), "{invalid:?}");
        }
        for good in [
            "SKILL.md",
            "scripts/check.py",
            "references/中文.md",
            "assets/icon.png",
            "COM10.txt",
        ] {
            validate_path(good).unwrap();
        }
    }
    #[test]
    fn metadata_is_bounded_declarative_and_rejects_ambiguous_requirements() {
        let mut m = Manifest::default();
        m.validate().unwrap();
        m.dependencies = vec![Dependency {
            name: "Python".into(),
            requirement: "3.11+; manually inspect scripts before use".into(),
        }];
        m.validate().unwrap();
        m.dependencies.push(Dependency {
            name: "python".into(),
            requirement: "other".into(),
        });
        assert!(m.validate().is_err());
        assert!(serde_json::from_str::<Manifest>(
            r#"{"schema":1,"version":"1","execute":"setup.ps1"}"#
        )
        .is_err());
        m.dependencies.clear();
        m.origin = Some("x\ny".into());
        assert!(m.validate().is_err());
    }
}
