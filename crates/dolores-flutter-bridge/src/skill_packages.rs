//! Reviewed package admission, retained storage and verified read-only resources.
use dolores_core::{skill_packages::*, SkillDocument};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};

pub mod bundles;
mod host;
pub(crate) mod resources;
pub use host::Request;
pub(super) use host::Review;

#[derive(Clone, Debug)]
pub struct InspectedPackage {
    pub document: SkillDocument,
    pub manifest: Manifest,
    pub archive_digest: String,
    pub content_digest: String,
    pub resources: Vec<Resource>,
    pub files: BTreeMap<String, Vec<u8>>,
    pub directories: Vec<String>,
}

/// The caller supplies the explicit intended skill name; SKILL.md must match it.
/// Keep the resulting bytes for review, rather than re-reading an untrusted ZIP
/// after consent and silently substituting changed resources.
pub fn inspect(name: &str, bytes: &[u8]) -> Result<InspectedPackage, String> {
    if !dolores_core::valid_skill_name(name) || bytes.is_empty() || bytes.len() > MAX_ARCHIVE_BYTES
    {
        return Err("Choose a valid skill name and a nonempty ZIP within 4 MiB.".into());
    }
    let problem = || {
        "Package ZIP is malformed, encrypted or unsupported. Export a normal ZIP with root SKILL.md and inspect it again.".to_string()
    };
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| problem())?;
    if zip.len() > MAX_PACKAGE_ENTRIES {
        return Err(
            "Package exceeds 128 archive entries. Remove unused files or split the package.".into(),
        );
    }
    let mut names = BTreeMap::new();
    let mut files = BTreeMap::new();
    let mut total = 0usize;
    let mut directories = BTreeMap::<String, String>::new();
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|_| problem())?;
        let raw = std::str::from_utf8(file.name_raw())
            .map_err(|_| "Package filenames must use UTF-8.")?;
        let directory = raw.ends_with('/');
        let path = if directory {
            &raw[..raw.len() - 1]
        } else {
            raw
        }
        .to_owned();
        validate_path(&path)?;
        if file.unix_mode().is_some_and(|m| {
            let kind = m & 0o170000;
            kind != 0 && kind != if directory { 0o040000 } else { 0o100000 }
        }) {
            return Err("Package links and special files are unsupported. Include ordinary files and directories only.".into());
        }
        let key = path.to_lowercase();
        let mut parent = if directory {
            Some(path.as_str())
        } else {
            path.rsplit_once('/').map(|(p, _)| p)
        };
        while let Some(p) = parent {
            if directories
                .insert(p.to_lowercase(), p.to_owned())
                .is_some_and(|old| old != p)
            {
                return Err("Package directory names differ only by case. Use one spelling for every parent directory.".into());
            }
            parent = p.rsplit_once('/').map(|(p, _)| p);
        }
        if names.insert(key.clone(), directory).is_some() {
            return Err("Package contains duplicate or case-colliding paths. Rename them and inspect again.".into());
        }
        // Catch both orders: a file cannot also contain another entry.
        if names.iter().any(|(other, is_dir)| {
            (!*is_dir && key.starts_with(&format!("{other}/")))
                || (!directory && other.starts_with(&format!("{key}/")))
        }) {
            return Err(
                "Package contains a file/directory collision. Rename the conflicting resource."
                    .into(),
            );
        }
        if directory {
            continue;
        }
        if files.len() == MAX_PACKAGE_FILES {
            return Err(
                "Package exceeds 64 files. Remove unused resources or split the package.".into(),
            );
        }
        let limit = match path.as_str() {
            "SKILL.md" => dolores_core::MAX_SKILL_BYTES,
            "dolores-package.json" => MAX_MANIFEST_BYTES,
            _ => MAX_RESOURCE_BYTES,
        };
        if file.size() > limit as u64 {
            return Err(format!("Package file {path} exceeds its {limit}-byte allowance. Reduce it and inspect again."));
        }
        let mut content = Vec::new();
        file.take((limit + 1) as u64)
            .read_to_end(&mut content)
            .map_err(|_| problem())?;
        total += content.len();
        if content.len() > limit || total > MAX_PACKAGE_BYTES {
            return Err("Expanded package exceeds its file allowance or 2 MiB total. Reduce the resources and inspect again.".into());
        }
        files.insert(path, content);
    }
    let skill = files.get("SKILL.md").ok_or("Package needs SKILL.md at the ZIP root. Export the skill folder contents, not its parent directory.")?;
    let document = dolores_tools_fs::parse_skill_document(
        name,
        String::from_utf8(skill.clone()).map_err(|_| "SKILL.md must be UTF-8.")?,
    )?;
    let manifest: Manifest = files.get("dolores-package.json").map(|b| serde_json::from_slice(b).map_err(|_| "Package manifest is invalid. Use schema 1 JSON with version, optional origin and dependencies.")).transpose()?.unwrap_or_default();
    manifest.validate()?;
    let mut hash = Sha256::new();
    hash.update(b"dolores-skill-package-v2\0");
    let directories: Vec<_> = directories
        .into_values()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    for path in &directories {
        hash.update(b"D");
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path.as_bytes());
    }
    let resources = files
        .iter()
        .map(|(path, content)| {
            hash.update(b"F");
            hash.update((path.len() as u64).to_le_bytes());
            hash.update(path.as_bytes());
            hash.update((content.len() as u64).to_le_bytes());
            hash.update(content);
            Resource {
                path: path.clone(),
                bytes: content.len(),
                digest: format!("{:x}", Sha256::digest(content)),
            }
        })
        .collect();
    Ok(InspectedPackage {
        document,
        manifest,
        archive_digest: format!("{:x}", Sha256::digest(bytes)),
        content_digest: format!("{:x}", hash.finalize()),
        resources,
        files,
        directories,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    const SKILL: &str = "---\nname: review\ndescription: Review a project\n---\nRead references/check.md; inspect scripts/check.py before execution.";
    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (path, bytes) in entries {
            zip.start_file(
                *path,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    #[test]
    fn complete_package_keeps_exact_resources_and_hashes_content_independent_of_zip_order() {
        let manifest = br#"{"schema":1,"version":"1.2.0","origin":"https://example.invalid/review","dependencies":[{"name":"Python","requirement":"3.11+"}]}"#;
        let entries = [
            ("SKILL.md", SKILL.as_bytes()),
            ("scripts/check.py", b"print('public fixture')".as_slice()),
            (
                "references/check.md",
                "Inspect changed bytes 世界".as_bytes(),
            ),
            ("assets/icon.bin", b"\0\x01\xff".as_slice()),
            ("dolores-package.json", manifest.as_slice()),
        ];
        let a = inspect("review", &archive(&entries)).unwrap();
        let b = inspect(
            "review",
            &archive(&entries.into_iter().rev().collect::<Vec<_>>()),
        )
        .unwrap();
        assert_eq!(a.content_digest, b.content_digest);
        assert_ne!(a.archive_digest, b.archive_digest);
        assert_eq!(a.files["assets/icon.bin"], b"\0\x01\xff");
        assert_eq!(a.document.text, SKILL);
        assert_eq!(a.manifest.version, "1.2.0");
        assert_eq!(a.resources.len(), 5);
        assert!(inspect("wrong", &archive(&entries)).is_err());
    }
    #[test]
    fn escaping_colliding_and_truncated_archives_refuse_without_publishing() {
        for path in ["../out", "/absolute", "C:/out", "a\\b", "NUL.txt", "a:b"] {
            assert!(
                inspect(
                    "review",
                    &archive(&[("SKILL.md", SKILL.as_bytes()), (path, b"x")])
                )
                .is_err(),
                "{path}"
            );
        }
        for paths in [
            ["scripts/a.py", "SCRIPTS/A.PY"],
            ["Scripts/a.py", "scripts/b.py"],
            ["scripts", "scripts/a.py"],
            ["scripts/a.py", "scripts"],
        ] {
            assert!(inspect(
                "review",
                &archive(&[
                    ("SKILL.md", SKILL.as_bytes()),
                    (paths[0], b"x"),
                    (paths[1], b"y")
                ])
            )
            .is_err());
        }
        let bytes = archive(&[("SKILL.md", SKILL.as_bytes())]);
        assert!(inspect("review", &bytes[..bytes.len() / 2]).is_err());
        assert!(inspect("review", &archive(&[("folder/SKILL.md", SKILL.as_bytes())])).is_err());
    }
    #[test]
    fn expanded_resources_and_malformed_metadata_are_bounded() {
        let big = vec![b'x'; MAX_RESOURCE_BYTES + 1];
        assert!(inspect(
            "review",
            &archive(&[("SKILL.md", SKILL.as_bytes()), ("assets/big", &big)])
        )
        .is_err());
        let block = vec![0; MAX_RESOURCE_BYTES];
        assert!(inspect(
            "review",
            &archive(&[
                ("SKILL.md", SKILL.as_bytes()),
                ("a", &block),
                ("b", &block),
                ("c", &block),
                ("d", &block)
            ])
        )
        .is_err());
        for metadata in [
            b"{".as_slice(),
            br#"{"schema":2,"version":"1"}"#.as_slice(),
            br#"{"schema":1,"version":"1","execute":"setup.ps1"}"#.as_slice(),
        ] {
            assert!(inspect(
                "review",
                &archive(&[
                    ("SKILL.md", SKILL.as_bytes()),
                    ("dolores-package.json", metadata)
                ])
            )
            .is_err());
        }
    }

    #[test]
    fn links_and_excessive_file_catalogues_are_refused() {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("SKILL.md", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(SKILL.as_bytes()).unwrap();
        zip.add_symlink(
            "scripts/check.py",
            "../../outside",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        let bytes = zip.finish().unwrap().into_inner();
        assert!(inspect("review", &bytes).unwrap_err().contains("links"));
        let names: Vec<_> = (0..64).map(|i| format!("resources/{i}")).collect();
        let mut entries = vec![("SKILL.md", SKILL.as_bytes())];
        entries.extend(names.iter().map(|n| (n.as_str(), b"x".as_slice())));
        assert!(inspect("review", &archive(&entries))
            .unwrap_err()
            .contains("64 files"));
    }
}
