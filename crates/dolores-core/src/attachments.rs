use serde::{Deserialize, Serialize};
pub const MAX_DRAFT_ATTACHMENTS: usize = 4;
pub const MAX_TEXT_ATTACHMENT_BYTES: usize = 64 * 1024;
pub const MAX_IMAGE_ATTACHMENT_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_ATTACHMENT_STORE_BYTES: usize = 64 * 1024 * 1024;
/// Local content part reference. It never includes a filesystem path or encoded bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttachmentRef {
    pub digest: String,
    pub name: String,
    pub mime: String,
    pub bytes: usize,
}
impl AttachmentRef {
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.digest.len() != 64
            || !self
                .digest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || self.name.is_empty()
            || self.name.len() > 255
            || self
                .name
                .chars()
                .any(|c| c.is_control() || "/\\".contains(c))
            || !["text/plain", "image/png", "image/jpeg"].contains(&self.mime.as_str())
            || self.bytes == 0
            || self.bytes
                > if self.is_image() {
                    MAX_IMAGE_ATTACHMENT_BYTES
                } else {
                    MAX_TEXT_ATTACHMENT_BYTES
                }
        {
            return Err(
                "Attachment reference is invalid. Remove it and attach the file again.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct AttachmentData {
    pub reference: AttachmentRef,
    pub data: Vec<u8>,
}
