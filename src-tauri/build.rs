fn main() {
    use sha2::{Digest, Sha256};
    let engine_name = if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        "quicklan-engine.exe"
    } else {
        "quicklan-engine"
    };
    let engine = std::path::Path::new("resources/engine").join(engine_name);
    println!("cargo:rerun-if-changed={}", engine.display());
    let digest = std::fs::read(&engine)
        .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
        .unwrap_or_default();
    println!("cargo:rustc-env=QUICKLAN_ENGINE_SHA256={digest}");
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_state",
            "get_local_endpoints",
            "probe_service",
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
