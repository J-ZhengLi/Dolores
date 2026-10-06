use sha2::{Digest, Sha256};
use std::process::Command;
use std::{fs, path::Path};

fn collect(root: &Path, directory: &Path, files: &mut Vec<String>) {
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory).expect("Read supported source directory") {
        let entry = entry.expect("Read source entry");
        let kind = entry.file_type().expect("Read source type");
        let path = entry.path();
        // Never follow links or include generated files, credentials or local profiles.
        if kind.is_dir()
            && !matches!(
                entry.file_name().to_str(),
                Some("target" | "build" | ".git" | ".dart_tool")
            )
        {
            collect(root, &path, files);
        } else if kind.is_file()
            && (matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("rs" | "dart" | "py")
            ) || path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml"))
        {
            files.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
            );
        }
    }
}

fn bundle(root: &Path) {
    let mut files = vec![
        "Cargo.toml".into(),
        "Cargo.lock".into(),
        "apps/dolores_flutter/pubspec.yaml".into(),
        "adapters/browser/worker.cjs".into(),
        // Workspace regressions execute this unchanged Node fixture by path.
        "scripts/mock-mcp.mjs".into(),
    ];
    for directory in [
        "crates",
        "apps/dolores_flutter/lib",
        "apps/dolores_flutter/test",
        "scripts",
    ] {
        collect(root, &root.join(directory), &mut files);
    }
    files.sort();
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut generated = String::from("pub const SOURCES: &[Source] = &[\n");
    let mut identity = Sha256::new();
    let mut total = 0;
    for (index, path) in files.iter().enumerate() {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
        let bytes = fs::read(root.join(path)).expect("Read supported source");
        let text = std::str::from_utf8(&bytes).expect("Supported source must be UTF-8");
        total += bytes.len();
        assert!(
            bytes.len() <= 1024 * 1024 && total <= 16 * 1024 * 1024,
            "Source bundle exceeds build bounds"
        );
        let digest = format!("sha256:{:x}", Sha256::digest(&bytes));
        identity.update(path.as_bytes());
        identity.update([0]);
        identity.update(digest.as_bytes());
        identity.update([0]);
        fs::write(
            output.join(format!("source-{index}.z")),
            miniz_oxide::deflate::compress_to_vec_zlib(&bytes, 6),
        )
        .unwrap();
        generated.push_str(&format!("Source {{ path: {path:?}, id: {digest:?}, lines: {}, bytes: {}, compressed: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/source-{index}.z\")) }},\n", text.lines().count(), bytes.len()));
    }
    generated.push_str(&format!(
        "];\npub const ID: &str = \"sha256:{:x}\";\n",
        identity.finalize()
    ));
    fs::write(output.join("source-bundle.rs"), generated).unwrap();
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    bundle(&root);
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs");
    // A staged repair below a checkout must not inherit that parent's Git identity.
    let revision = root
        .join(".git")
        .exists()
        .then(|| {
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(root)
                .output()
        })
        .transpose()
        .ok()
        .flatten()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=DOLORES_BUILD_REVISION={revision}");
}
