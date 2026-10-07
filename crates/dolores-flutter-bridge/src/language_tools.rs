//! Explicit pinned installation. Archives are verified before extraction; no scripts run.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
const TLS: &str = "6.0.1";
const TS: &str = "6.0.3";
const RA: &str = "2026-10-05";
pub(super) struct Install {
    pub id: String,
    pub cancel: CancellationToken,
    pub state: Arc<Mutex<Value>>,
    pub task: tokio::task::JoinHandle<()>,
}
pub(super) fn info() -> Value {
    json!({"typescript":{"server":TLS,"compiler":TS,"source":"npm registry","downloadLimitMiB":64},"rust":{"version":RA,"source":"rust-lang/rust-analyzer","platform":"Windows x64","downloadLimitMiB":64},"note":"Install only after clicking Install. Pinned hashes are checked before extraction; no package scripts run. Cancellation or failure retains the previous setup."})
}
fn canceled(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Installation canceled. Previous tools remain.".into())
    } else {
        Ok(())
    }
}
fn update(state: &Arc<Mutex<Value>>, stage: &str, bytes: usize) {
    if let Ok(mut s) = state.lock() {
        *s = json!({"state":stage,"bytes":bytes});
    }
}
fn verify(bytes: &[u8], hash: &str) -> Result<(), String> {
    let actual = if hash.starts_with("sha512-") {
        format!("sha512-{}", STANDARD.encode(Sha512::digest(bytes)))
    } else {
        format!("sha256:{:x}", Sha256::digest(bytes))
    };
    if actual == hash {
        Ok(())
    } else {
        Err(
            "Pinned artifact checksum failed. Previous tools remain; retry the verified version."
                .into(),
        )
    }
}
async fn download(
    client: &reqwest::Client,
    url: &str,
    hash: &str,
    cancel: &CancellationToken,
    state: &Arc<Mutex<Value>>,
) -> Result<Vec<u8>, String> {
    canceled(cancel)?;
    let mut response = tokio::select! {_=cancel.cancelled()=>return Err("Installation canceled. Previous tools remain.".into()),v=client.get(url).send()=>v.map_err(|_|"Language download unavailable. Check connection and retry; previous tools remain.")?};
    if !response.status().is_success() {
        return Err(
            "Language download unavailable. Retry when online; previous tools remain.".into(),
        );
    }
    let mut bytes = vec![];
    loop {
        let chunk = tokio::select! {_=cancel.cancelled()=>return Err("Installation canceled. Previous tools remain.".into()),v=response.chunk()=>v.map_err(|_|"Language download interrupted. Previous tools remain.")?};
        let Some(chunk) = chunk else { break };
        if bytes.len() + chunk.len() > 64 * 1024 * 1024 {
            return Err("Language download exceeds 64 MiB. Previous tools remain.".into());
        }
        bytes.extend_from_slice(&chunk);
        update(state, "downloading", bytes.len());
    }
    update(state, "verifying", bytes.len());
    verify(&bytes, hash)?;
    Ok(bytes)
}
fn unpack_tar(bytes: &[u8], dest: &Path, cancel: &CancellationToken) -> Result<(), String> {
    let mut gzip = flate2::read::GzDecoder::new(bytes).take(64 * 1024 * 1024 + 1);
    let mut raw = vec![];
    gzip.read_to_end(&mut raw)
        .map_err(|_| "Language archive is damaged.")?;
    if raw.len() > 64 * 1024 * 1024 {
        return Err("Language archive exceeds 64 MiB unpacked.".into());
    }
    let mut archive = tar::Archive::new(raw.as_slice());
    let mut count = 0;
    let mut total = 0;
    for entry in archive
        .entries()
        .map_err(|_| "Language archive is invalid.")?
    {
        canceled(cancel)?;
        let mut entry = entry.map_err(|_| "Language archive entry is invalid.")?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err("Linked language archive entries are refused.".into());
        }
        let path = entry
            .path()
            .map_err(|_| "Language archive path is invalid.")?;
        let relative = path
            .strip_prefix("package")
            .map_err(|_| "Language archive package root is invalid.")?;
        if relative
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
            || relative.as_os_str().is_empty()
        {
            return Err("Language archive escaped its package folder.".into());
        }
        count += 1;
        total += entry.size();
        if count > 2048 || total > 64 * 1024 * 1024 || entry.size() > 32 * 1024 * 1024 {
            return Err("Language archive exceeds file allowances.".into());
        }
        let out = dest.join(relative);
        std::fs::create_dir_all(out.parent().unwrap())
            .map_err(|_| "Language staging folder unavailable.")?;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out)
            .map_err(|_| "Language file already exists or could not be staged.")?;
        std::io::copy(&mut entry, &mut f).map_err(|_| "Language file could not be staged.")?;
        f.sync_all()
            .map_err(|_| "Language file could not be synced.")?;
    }
    Ok(())
}
fn unpack_rust(bytes: &[u8], dest: &Path, cancel: &CancellationToken) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|_| "Rust language archive is invalid.")?;
    if !(1..=2).contains(&archive.len()) {
        return Err("Rust language archive has unexpected entries.".into());
    }
    for index in 0..archive.len() {
        let member = archive
            .by_index(index)
            .map_err(|_| "Rust archive entry unavailable.")?;
        match member.name() {
            "rust-analyzer.exe" if member.size() <= 128 * 1024 * 1024 => {}
            // The pinned upstream Windows archive also ships optional debug
            // symbols. Validate their name/bound but do not install them.
            "rust_analyzer.pdb" if member.size() <= 32 * 1024 * 1024 => {}
            _ => return Err("Rust archive has an unexpected member.".into()),
        }
    }
    let mut entry = archive
        .by_name("rust-analyzer.exe")
        .map_err(|_| "Rust archive entry unavailable.")?;
    if entry.name() != "rust-analyzer.exe" || entry.size() > 128 * 1024 * 1024 {
        return Err("Rust archive has an unexpected executable.".into());
    }
    canceled(cancel)?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest.join("rust-analyzer.exe"))
        .map_err(|_| "Rust executable could not be staged.")?;
    std::io::copy(&mut entry, &mut f).map_err(|_| "Rust executable could not be extracted.")?;
    f.sync_all()
        .map_err(|_| "Rust executable could not be synced.")?;
    Ok(())
}
pub(super) fn installed(managed: &Path, language: &str) -> Result<Option<PathBuf>, String> {
    let pointer = managed.join("installed.json");
    if !pointer.exists() {
        return Ok(None);
    }
    let value: Value = dolores_native_update::json(&pointer)?;
    let Some(item) = value.get(language) else {
        return Ok(None);
    };
    let folder = item["folder"]
        .as_str()
        .ok_or("Installed language receipt is invalid.")?;
    if folder.contains(['/', '\\']) || folder.contains("..") {
        return Err("Installed language folder is invalid.".into());
    }
    let root = managed.join(folder);
    let bundle: dolores_native_update::Bundle =
        dolores_native_update::json(&root.join("receipt.json"))?;
    bundle.validate()?;
    for file in &bundle.files {
        let path = root.join(&file.path);
        if file.path.contains("..")
            || Path::new(&file.path).is_absolute()
            || dolores_native_update::file_hash(&path)? != file.id
        {
            return Err(
                "Installed language files changed. Reinstall the verified tools; buffers remain."
                    .into(),
            );
        }
    }
    Ok(Some(root))
}
pub(super) async fn install(
    managed: PathBuf,
    language: String,
    cancel: CancellationToken,
    state: Arc<Mutex<Value>>,
) -> Result<(), String> {
    if !["typescript", "rust"].contains(&language.as_str()) {
        return Err("Unsupported language install.".into());
    }
    if language == "rust" && !cfg!(all(windows, target_arch = "x86_64")) {
        return Err("Managed Rust installation is qualified on Windows x64 only. Existing servers remain usable.".into());
    }
    std::fs::create_dir_all(&managed).map_err(|_| "Private language storage unavailable.")?;
    dolores_native_update::safe_path(&managed)?;
    let stage = tempfile::Builder::new()
        .prefix("install-")
        .tempdir_in(&managed)
        .map_err(|_| "Language staging unavailable.")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("Dolores-language-tools")
        .build()
        .map_err(|_| "Language download client unavailable.")?;
    if language == "typescript" {
        let bytes=download(&client,"https://registry.npmjs.org/typescript-language-server/-/typescript-language-server-6.0.1.tgz","sha512-c5hEHM/7rdFRbQWoHXuddyJ53oF0M2dKeXQOYdwUek556oarltKeumKJGLp/ZMcGV7gW30mGn1MR3V5GDiLiKg==",&cancel,&state).await?;
        update(&state, "extracting", bytes.len());
        unpack_tar(
            &bytes,
            &stage.path().join("node_modules/typescript-language-server"),
            &cancel,
        )?;
        let bytes=download(&client,"https://registry.npmjs.org/typescript/-/typescript-6.0.3.tgz","sha512-y2TvuxSZPDyQakkFRPZHKFm+KKVqIisdg9/CZwm9ftvKXLP8NRWj38/ODjNbr43SsoXqNuAisEf1GdCxqWcdBw==",&cancel,&state).await?;
        update(&state, "extracting", bytes.len());
        unpack_tar(
            &bytes,
            &stage.path().join("node_modules/typescript"),
            &cancel,
        )?;
        if !stage
            .path()
            .join("node_modules/typescript-language-server/lib/cli.mjs")
            .is_file()
            || !stage
                .path()
                .join("node_modules/typescript/lib/tsserver.js")
                .is_file()
        {
            return Err(
                "Pinned TypeScript package layout is incompatible. Previous tools remain.".into(),
            );
        }
    } else {
        let bytes=download(&client,"https://github.com/rust-lang/rust-analyzer/releases/download/2026-10-05/rust-analyzer-x86_64-pc-windows-msvc.zip","sha256:5a9d499aa26d835406cbccfc52f12bce8286733c5bfe62d79f0ed87cae2e64b9",&cancel,&state).await?;
        update(&state, "extracting", bytes.len());
        unpack_rust(&bytes, stage.path(), &cancel)?;
    }
    canceled(&cancel)?;
    update(&state, "activating", 0);
    let bundle = dolores_native_update::manifest(stage.path())?;
    dolores_native_update::save(&stage.path().join("receipt.json"), &bundle)?;
    // Unique immutable versions prevent an interrupted replacement overwriting old files.
    let folder = format!(
        "{language}-{}-{}",
        if language == "rust" { RA } else { TLS },
        uuid::Uuid::new_v4()
    );
    let dest = managed.join(&folder);
    std::fs::rename(stage.path(), &dest)
        .map_err(|_| "Verified language folder could not be retained. Previous tools remain.")?;
    let pointer = managed.join("installed.json");
    let mut value: Value = if pointer.exists() {
        dolores_native_update::json(&pointer)?
    } else {
        json!({})
    };
    canceled(&cancel)?;
    value[&language] = json!({"folder":folder,"version":if language=="rust"{RA}else{TLS},"compiler":if language=="typescript"{TS}else{""}});
    dolores_native_update::save(&pointer, &value)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rust_archive_accepts_optional_symbols_and_refuses_other_members() {
        fn archive(extra: &str) -> Vec<u8> {
            use std::io::Write;
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            zip.start_file(
                "rust-analyzer.exe",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(b"fixture executable").unwrap();
            zip.start_file(extra, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"fixture symbols").unwrap();
            zip.finish().unwrap().into_inner()
        }
        let good = tempfile::tempdir().unwrap();
        unpack_rust(
            &archive("rust_analyzer.pdb"),
            good.path(),
            &CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(std::fs::read_dir(good.path()).unwrap().count(), 1);
        let bad = tempfile::tempdir().unwrap();
        assert!(unpack_rust(
            &archive("../outside.exe"),
            bad.path(),
            &CancellationToken::new()
        )
        .is_err());
        assert_eq!(std::fs::read_dir(bad.path()).unwrap().count(), 0);
    }
    #[tokio::test]
    async fn canceled_install_and_offline_download_never_replace_prior_setup() {
        let dir = tempfile::tempdir().unwrap();
        let pointer = dir.path().join("installed.json");
        std::fs::write(&pointer, b"previous setup").unwrap();
        let canceled = CancellationToken::new();
        canceled.cancel();
        let state = Arc::new(Mutex::new(json!({})));
        assert!(install(
            dir.path().to_owned(),
            "typescript".into(),
            canceled,
            state.clone()
        )
        .await
        .unwrap_err()
        .contains("canceled"));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();
        assert!(download(
            &client,
            &format!("http://{address}/offline"),
            "sha256:bad",
            &CancellationToken::new(),
            &state
        )
        .await
        .unwrap_err()
        .contains("unavailable"));
        assert_eq!(std::fs::read(pointer).unwrap(), b"previous setup");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn checksum_cancel_and_bad_archive_keep_installed_pointer() {
        let dir = tempfile::tempdir().unwrap();
        let pointer = dir.path().join("installed.json");
        std::fs::write(&pointer, b"previous").unwrap();
        assert!(verify(b"tampered", "sha256:bad").is_err());
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(canceled(&cancel).is_err());
        assert!(unpack_tar(b"bad", dir.path(), &CancellationToken::new()).is_err());
        assert_eq!(std::fs::read(pointer).unwrap(), b"previous");
    }
}
