use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SearchProvider {
    #[default]
    Mwmbl,
    Brave,
    Searxng,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WebConfiguration {
    pub revision: u32,
    pub enabled: bool,
    pub provider: SearchProvider,
    pub endpoint: Option<String>,
    pub credential_id: Option<String>,
    #[serde(default)]
    pub retired_credentials: Vec<String>,
}
impl Default for WebConfiguration {
    fn default() -> Self {
        Self {
            revision: 0,
            enabled: true,
            provider: SearchProvider::Mwmbl,
            endpoint: None,
            credential_id: None,
            retired_credentials: vec![],
        }
    }
}
impl WebConfiguration {
    pub fn validate(&self) -> Result<(), String> {
        if self.retired_credentials.len() > 8
            || self
                .credential_id
                .iter()
                .chain(self.retired_credentials.iter())
                .any(|id| {
                    id.is_empty()
                        || id.len() > 128
                        || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
                })
        {
            return Err("Web credential references are invalid. Open Web search settings.".into());
        }
        match (&self.provider, &self.endpoint) {
            (SearchProvider::Searxng, Some(url))
                if url.starts_with("https://")
                    && url.len() <= 1024
                    && !url.chars().any(char::is_control) =>
            {
                Ok(())
            }
            (SearchProvider::Mwmbl | SearchProvider::Brave, None) => Ok(()),
            _ => Err(
                "Choose a public HTTPS SearXNG search endpoint, or a built-in search provider."
                    .into(),
            ),
        }
    }
}
