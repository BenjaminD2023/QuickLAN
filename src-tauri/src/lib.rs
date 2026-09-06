mod commands;
use quicklan_core::{
    app::App,
    error::{Error, Result},
    storage::OsStore,
};
use std::sync::Mutex;
use tauri::Manager;
pub struct ManagedApp(Mutex<Result<App<OsStore>>>);
impl ManagedApp {
    fn with<T>(&self, f: impl FnOnce(&mut App<OsStore>) -> Result<T>) -> Result<T> {
        let mut state = self.0.lock().map_err(|_| Error::CoreFailed)?;
        match &mut *state {
            Ok(app) => f(app),
            Err(error) => Err(error.clone()),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let state = app
                .path()
                .app_data_dir()
                .map_err(|_| Error::UnsafePath)
                .and_then(OsStore::new)
                .and_then(App::new);
            let state = state.map(|state| {
                let engine_name = if cfg!(windows) {
                    "quicklan-engine.exe"
                } else {
                    "quicklan-engine"
                };
                match app.path().resource_dir() {
                    Ok(root) => {
                        state.with_runtime(Box::new(quicklan_runtime::DesktopRuntime::new(
                            root.join("resources/engine").join(engine_name),
                            env!("QUICKLAN_ENGINE_SHA256").to_owned(),
                        )))
                    }
                    Err(_) => state,
                }
            });
            app.manage(ManagedApp(Mutex::new(state)));
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                // No close-to-tray/background networking in this build.
                let _ = window.state::<ManagedApp>().with(|app| app.disconnect());
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::probe_service,
            commands::update_settings,
            commands::create_network,
            commands::preview_invitation,
            commands::accept_invitation,
            commands::cancel_invitation,
            commands::rename_network,
            commands::forget_network,
            commands::replace_credentials,
            commands::connect_network,
            commands::disconnect_network,
            commands::save_preferences,
            commands::get_diagnostics,
            commands::copy_diagnostics,
            commands::copy_invitation,
            commands::copy_virtual_ip,
            commands::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("QuickLAN could not start its desktop runtime");
}
