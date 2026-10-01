mod commands;
use commands::AppState;
use dolores_store_sqlite::SqliteStore;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = match std::env::var_os("DOLORES_DATA_DIR") {
                Some(value) => {
                    let path = std::path::PathBuf::from(value);
                    if !path.is_absolute() {
                        return Err(std::io::Error::other(
                            "DOLORES_DATA_DIR must be an absolute path",
                        )
                        .into());
                    }
                    path
                }
                None => app.path().app_data_dir()?,
            };
            std::fs::create_dir_all(&directory)?;
            let store =
                SqliteStore::open(&directory.join("dolores.db")).map_err(std::io::Error::other)?;
            app.manage(AppState {
                store: Arc::new(store),
                provider: Mutex::new(None),
                active: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::create_session,
            commands::get_messages,
            commands::delete_session,
            commands::configure_connection,
            commands::generate,
            commands::cancel_run
        ])
        .run(tauri::generate_context!())
        .expect("Could not start Dolores desktop");
}
