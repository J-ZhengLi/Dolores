use crate::Engine;
use dolores_core::ExperimentalPreferences;
use serde_json::{json, Value};

pub(crate) struct PowerRequest {
    #[cfg(windows)]
    handle: usize,
}
impl PowerRequest {
    fn create() -> Result<Self, String> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::{
                Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
                System::{
                    Power::*,
                    Threading::{
                        POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0,
                    },
                },
            };
            let mut reason: Vec<u16> = "Dolores experimental keep-awake preference"
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let context = REASON_CONTEXT {
                Version: 0,
                Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
                Reason: REASON_CONTEXT_0 {
                    SimpleReasonString: reason.as_mut_ptr(),
                },
            };
            // A process-owned power request: no synthetic input or security setting changes.
            unsafe {
                let handle = PowerCreateRequest(&context);
                if handle == INVALID_HANDLE_VALUE || handle.is_null() {
                    return Err(
                        "Windows could not create a keep-awake request. Retry or leave it off."
                            .into(),
                    );
                }
                if PowerSetRequest(handle, PowerRequestSystemRequired) == 0 {
                    CloseHandle(handle);
                    return Err("Windows refused the system keep-awake request.".into());
                }
                if PowerSetRequest(handle, PowerRequestDisplayRequired) == 0 {
                    PowerClearRequest(handle, PowerRequestSystemRequired);
                    CloseHandle(handle);
                    return Err("Windows refused the display keep-awake request.".into());
                }
                Ok(Self {
                    handle: handle as usize,
                })
            }
        }
        #[cfg(not(windows))]
        {
            Err("Keep-awake is available on Windows only.".into())
        }
    }
}
impl Drop for PowerRequest {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::{Foundation::CloseHandle, System::Power::*};
            let h = self.handle as windows_sys::Win32::Foundation::HANDLE;
            PowerClearRequest(h, PowerRequestDisplayRequired);
            PowerClearRequest(h, PowerRequestSystemRequired);
            CloseHandle(h);
        }
    }
}
impl Engine {
    pub(crate) fn experimental_view(&self) -> Result<Value, String> {
        let active = self
            .keep_awake
            .lock()
            .map_err(|_| "Keep-awake state is unavailable.")?
            .is_some();
        Ok(
            json!({"preferences":self.store.experimental_preferences()?,"windows":cfg!(windows),"keepAwakeActive":active,
            "multipleWindowCapability": {
                "available": false,
                "status": "held",
                "reason": "Additional windows are unavailable in this build because their performance checks did not pass. Use split views in Folders, Source Control or Terminal.",
                "qualification": "19.1: added idle CPU exceeded the frozen less-than-1% gate; 19.2/19.3 held"
            }}),
        )
    }
    pub(crate) fn save_experimental(
        &self,
        value: ExperimentalPreferences,
    ) -> Result<Value, String> {
        let mut active = self
            .keep_awake
            .lock()
            .map_err(|_| "Keep-awake state is unavailable.")?;
        if self.store.experimental_preferences()?.revision != value.revision {
            return Err("Experimental preferences changed. Refresh before retrying.".into());
        }
        let candidate = if value.prevent_windows_from_locked && active.is_none() {
            Some(PowerRequest::create()?)
        } else {
            None
        };
        self.store.save_experimental_preferences(&value)?;
        if !value.prevent_windows_from_locked {
            active.take();
        } else if candidate.is_some() {
            *active = candidate;
        }
        drop(active);
        self.experimental_view()
    }
    pub(crate) fn restore_keep_awake(&self) {
        if self
            .store
            .experimental_preferences()
            .is_ok_and(|p| p.prevent_windows_from_locked)
        {
            if let Ok(request) = PowerRequest::create() {
                if let Ok(mut slot) = self.keep_awake.lock() {
                    *slot = Some(request);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connection::testing::MemoryCredentials, Command};
    use dolores_core::SessionStore;
    use dolores_store_sqlite::SqliteStore;
    use std::sync::Arc;
    #[test]
    fn preferences_have_no_default_power_request_and_stale_write_is_refused() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(store.clone(), Arc::new(MemoryCredentials::default())).unwrap();
        let view = engine.experimental_view().unwrap();
        assert_eq!(view["preferences"]["multipleWindow"], true);
        assert_eq!(view["multipleWindowCapability"]["available"], false);
        assert_eq!(view["multipleWindowCapability"]["status"], "held");
        assert_eq!(
            engine.experimental_view().unwrap()["keepAwakeActive"],
            false
        );
        let mut p = store.experimental_preferences().unwrap();
        p.multiple_window = false;
        assert_eq!(
            engine.save_experimental(p.clone()).unwrap()["preferences"]["multipleWindow"],
            false
        );
        assert!(engine.save_experimental(p).is_err());
        let held = engine.experimental_view().unwrap();
        assert_eq!(held["preferences"]["multipleWindow"], false);
        assert_eq!(
            held["multipleWindowCapability"],
            view["multipleWindowCapability"]
        );
        assert_eq!(store.experimental_preferences().unwrap().revision, 1);
        engine.call(Command::Shutdown).unwrap();
        assert!(engine.keep_awake.lock().unwrap().is_none());
    }
    #[test]
    fn held_window_preference_restores_without_starting_processes() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let credentials = Arc::new(MemoryCredentials::default());
        let engine = Engine::new(store.clone(), credentials.clone()).unwrap();
        let mut prefs = store.experimental_preferences().unwrap();
        prefs.multiple_window = false;
        engine.save_experimental(prefs).unwrap();
        engine.call(Command::Shutdown).unwrap();
        drop(engine);
        let restored = Engine::new(store.clone(), credentials).unwrap();
        assert_eq!(
            restored.experimental_view().unwrap()["preferences"]["multipleWindow"],
            false
        );
        assert_eq!(
            restored.experimental_view().unwrap()["multipleWindowCapability"]["available"],
            false
        );
        let mut prefs = store.experimental_preferences().unwrap();
        prefs.multiple_window = true;
        restored.save_experimental(prefs).unwrap();
        assert_eq!(
            restored.experimental_view().unwrap()["preferences"]["multipleWindow"],
            true
        );
        assert!(restored.keep_awake.lock().unwrap().is_none());
        assert!(restored
            .terminal_call(crate::terminal::Request::List)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty());
        restored.call(Command::Shutdown).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn process_owned_windows_request_releases_on_disable_and_shutdown() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(store.clone(), Arc::new(MemoryCredentials::default())).unwrap();
        let mut p = store.experimental_preferences().unwrap();
        p.prevent_windows_from_locked = true;
        assert_eq!(
            engine.save_experimental(p).unwrap()["keepAwakeActive"],
            true
        );
        let mut p = store.experimental_preferences().unwrap();
        p.prevent_windows_from_locked = false;
        assert_eq!(
            engine.save_experimental(p).unwrap()["keepAwakeActive"],
            false
        );
        let mut p = store.experimental_preferences().unwrap();
        p.prevent_windows_from_locked = true;
        engine.save_experimental(p).unwrap();
        engine.call(Command::Shutdown).unwrap();
        assert!(engine.keep_awake.lock().unwrap().is_none());
    }
}
