use crate::ManagedApp;
use quicklan_core::{
    app::{AppView, JoinPreview},
    diagnostics::DiagnosticReport,
    error::{Error, Result},
    lifecycle::Snapshot,
    model::{Bootstrap, Network, Policy, Preferences},
};
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn get_state(state: State<'_, ManagedApp>) -> Result<AppView> {
    state.with(|a| {
        a.refresh()?;
        Ok(a.view())
    })
}
#[tauri::command]
pub async fn create_network(
    state: State<'_, ManagedApp>,
    label: String,
    nickname: String,
    subnet: String,
    policy: Policy,
    bootstrap: Vec<Bootstrap>,
    assistance_accepted: bool,
) -> Result<Network> {
    state.with(|a| {
        a.create_with_nickname(
            label,
            subnet,
            policy,
            bootstrap,
            assistance_accepted,
            nickname,
        )
    })
}
#[tauri::command]
pub async fn preview_invitation(
    state: State<'_, ManagedApp>,
    token: String,
) -> Result<JoinPreview> {
    let token = zeroize::Zeroizing::new(token);
    state.with(|a| a.preview(&token))
}
#[tauri::command]
pub async fn accept_invitation(
    state: State<'_, ManagedApp>,
    ticket: String,
    trusted: bool,
    assistance_accepted: bool,
) -> Result<Network> {
    state.with(|a| a.accept(&ticket, trusted, assistance_accepted))
}
#[tauri::command]
pub async fn cancel_invitation(state: State<'_, ManagedApp>) -> Result<()> {
    state.with(|a| {
        a.cancel_preview();
        Ok(())
    })
}
#[tauri::command]
pub async fn rename_network(state: State<'_, ManagedApp>, id: String, label: String) -> Result<()> {
    state.with(|a| a.rename(&id, label))
}
#[tauri::command]
pub async fn forget_network(state: State<'_, ManagedApp>, id: String) -> Result<()> {
    state.with(|a| a.forget(&id))
}
#[tauri::command]
pub async fn replace_credentials(state: State<'_, ManagedApp>, id: String) -> Result<Network> {
    state.with(|a| a.replace_credentials(&id))
}
#[tauri::command]
pub async fn connect_network(state: State<'_, ManagedApp>, id: String) -> Result<Snapshot> {
    state.with(|a| a.connect(&id))
}
#[tauri::command]
pub async fn disconnect_network(state: State<'_, ManagedApp>) -> Result<Snapshot> {
    state.with(|a| a.disconnect())
}
#[tauri::command]
pub async fn save_preferences(
    state: State<'_, ManagedApp>,
    preferences: Preferences,
) -> Result<()> {
    state.with(|a| a.save_preferences(preferences))
}
#[tauri::command]
pub async fn get_diagnostics(state: State<'_, ManagedApp>) -> Result<DiagnosticReport> {
    state.with(|a| Ok(a.diagnostics()))
}
#[tauri::command]
pub async fn copy_invitation(
    state: State<'_, ManagedApp>,
    handle: tauri::AppHandle,
    id: String,
) -> Result<()> {
    let token = state.with(|a| a.invitation(&id))?;
    handle
        .clipboard()
        .write_text(token.as_str())
        .map_err(|_| Error::StorageUnavailable)
}
#[tauri::command]
pub async fn copy_diagnostics(
    state: State<'_, ManagedApp>,
    handle: tauri::AppHandle,
) -> Result<()> {
    let report = state.with(|a| Ok(a.diagnostics()))?;
    handle
        .clipboard()
        .write_text(serde_json::to_string_pretty(&report).map_err(|_| Error::CoreFailed)?)
        .map_err(|_| Error::StorageUnavailable)
}
#[tauri::command]
pub async fn copy_virtual_ip(
    state: State<'_, ManagedApp>,
    handle: tauri::AppHandle,
    ip: String,
) -> Result<()> {
    state.with(|a| {
        let snapshot = a.view().connection;
        if snapshot.virtual_ip.as_deref() != Some(&ip)
            && !snapshot
                .peers
                .iter()
                .any(|p| p.virtual_ip.as_deref() == Some(&ip))
        {
            return Err(Error::InvalidService);
        }
        Ok(())
    })?;
    handle
        .clipboard()
        .write_text(ip)
        .map_err(|_| Error::StorageUnavailable)
}
#[tauri::command]
pub async fn quit_app(state: State<'_, ManagedApp>, handle: tauri::AppHandle) -> Result<()> {
    // Allow quit even when credential storage is locked; no core can run then.
    let _ = state.with(|a| a.disconnect());
    handle.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, ManagedApp>,
    id: String,
    policy: Policy,
    bootstrap: Vec<Bootstrap>,
    assistance_accepted: bool,
) -> Result<()> {
    state.with(|a| a.update_settings(&id, policy, bootstrap, assistance_accepted))
}

#[tauri::command]
pub async fn probe_service(
    state: State<'_, ManagedApp>,
    peer_id: String,
    port: u16,
) -> Result<quicklan_core::service::ProbeResult> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUSY: AtomicBool = AtomicBool::new(false);
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err(Error::Busy);
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            BUSY.store(false, Ordering::SeqCst);
        }
    }
    let guard = Guard;
    let endpoint = state.with(|app| app.service_endpoint(&peer_id, port))?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        quicklan_core::service::probe(endpoint)
    })
    .await
    .map_err(|_| Error::CoreFailed)
}
