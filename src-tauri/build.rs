fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_state",
            "update_settings",
            "create_network",
            "preview_invitation",
            "accept_invitation",
            "cancel_invitation",
            "rename_network",
            "forget_network",
            "replace_credentials",
            "connect_network",
            "disconnect_network",
            "save_preferences",
            "get_diagnostics",
            "copy_diagnostics",
            "copy_invitation",
            "copy_virtual_ip",
            "quit_app",
        ]),
    ))
    .expect("QuickLAN Tauri build failed");
}
