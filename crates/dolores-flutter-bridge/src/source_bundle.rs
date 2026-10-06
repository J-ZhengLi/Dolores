//! Build-pinned, read-only source. No runtime checkout or background service.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub struct Source {
    pub path: &'static str,
    pub id: &'static str,
    pub lines: usize,
    pub bytes: usize,
    pub compressed: &'static [u8],
}
include!(concat!(env!("OUT_DIR"), "/source-bundle.rs"));

pub fn find(path: &str) -> Result<&'static Source, String> {
    SOURCES.iter().find(|s| s.path == path).ok_or_else(||
        "Matching source is unavailable. List the running build's source bundle; your task and evidence remain.".into())
}
pub fn text(source: &Source) -> Result<String, String> {
    let bytes = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(source.compressed, source.bytes)
        .map_err(|_| "Bundled source could not be decoded. Keep the diagnosis and install a matching normal build.")?;
    if bytes.len() != source.bytes || format!("sha256:{:x}", Sha256::digest(&bytes)) != source.id {
        return Err(
            "Bundled source identity changed. Diagnosis remains; do not patch this source.".into(),
        );
    }
    String::from_utf8(bytes).map_err(|_| "Bundled source is not UTF-8.".into())
}
pub fn entry(source: &Source) -> Value {
    json!({"id":source.path,"path":source.path,"sourceId":source.id,"lines":source.lines,"bytes":source.bytes})
}
pub fn summary() -> Value {
    json!({"bundleId":ID,"files":SOURCES.len(),"compressedBytes":SOURCES.iter().map(|s|s.compressed.len()).sum::<usize>(),"sourceBytes":SOURCES.iter().map(|s|s.bytes).sum::<usize>(),"origin":"bundled with running build","loadedLazily":true})
}
