//! OS credential plugin. Never uses keyring's mock backend as a production fallback.
use dolores_core::{CredentialStore, PluginDescriptor};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct OsCredentialStore {
    scope: String,
}
impl OsCredentialStore {
    pub fn new(directory: &Path) -> Result<Self, String> {
        let canonical = directory
            .canonicalize()
            .map_err(|_| "Could not identify the connection workspace.")?;
        let identity = canonical.to_string_lossy().into_owned();
        #[cfg(windows)]
        let identity = identity.to_lowercase();
        Ok(Self {
            scope: format!("{:x}", Sha256::digest(identity.as_bytes())),
        })
    }
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    fn entry(&self, id: &str) -> Result<keyring::Entry, String> {
        uuid::Uuid::parse_str(id).map_err(|_| "Saved connection identity is invalid.")?;
        keyring::Entry::new(
            "dev.dolores.desktop.connection",
            &format!("{}:{id}", self.scope),
        )
        .map_err(|_| unavailable())
    }
}
fn unavailable() -> String {
    "Secure storage is unavailable. Unlock it and retry, or turn off Remember connection to use this launch only.".into()
}
impl CredentialStore for OsCredentialStore {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "dolores.credentials.os",
            kind: "credentials",
            api_version: 1,
        }
    }
    fn read(&self, id: &str) -> Result<Option<String>, String> {
        #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
        {
            match self.entry(id)?.get_password() {
                Ok(value) => Ok(Some(value)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(_) => Err(unavailable()),
            }
        }
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            let _ = id;
            Err(unavailable())
        }
    }
    fn write(&self, id: &str, secret: &str) -> Result<(), String> {
        #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
        {
            self.entry(id)?
                .set_password(secret)
                .map_err(|_| unavailable())
        }
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            let _ = (id, secret);
            Err(unavailable())
        }
    }
    fn delete(&self, id: &str) -> Result<(), String> {
        #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
        {
            match self.entry(id)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(_) => Err(unavailable()),
            }
        }
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            let _ = id;
            Err(unavailable())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Only an explicit local invocation writes a generated test entry. CI does
    // not assume a signed-in, unlocked OS credential store.
    #[test]
    #[ignore = "requires an unlocked native OS credential store"]
    fn native_store_round_trip_is_scoped_and_deleted() {
        let directory =
            std::env::temp_dir().join(format!("dolores-vault-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let other_directory = directory.join("other-workspace");
        std::fs::create_dir(&other_directory).unwrap();
        let store = OsCredentialStore::new(&directory).unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let result = (|| {
            store.write(&id, "dolores-generated-test-value")?;
            let reopened = OsCredentialStore::new(&directory)?;
            if reopened.read(&id)? != Some("dolores-generated-test-value".into()) {
                return Err("Generated test entry did not round-trip".into());
            }
            let other = OsCredentialStore::new(&other_directory)?;
            if other.read(&id)?.is_some() {
                return Err("Another workspace could read the generated entry".into());
            }
            reopened.delete(&id)?;
            if reopened.read(&id)?.is_some() {
                return Err("Generated test entry was not deleted".into());
            }
            Ok::<_, String>(())
        })();
        let cleanup = store.delete(&id);
        std::fs::remove_dir(&other_directory).unwrap();
        std::fs::remove_dir(&directory).unwrap();
        result.unwrap();
        cleanup.unwrap();
    }
}
