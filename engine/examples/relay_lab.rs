//! Disposable namespace relay for policy integration tests. Never packaged.
use easytier::{
    common::config::{ConfigLoader, TomlConfigLoader, process_secure_mode_cfg},
    instance::instance::Instance,
    proto::common::SecureModeConfig,
};
#[tokio::main]
async fn main() {
    assert_eq!(std::env::var("CI").as_deref(), Ok("true"));
    assert!(quicklan_ipc::is_elevated());
    let cfg = TomlConfigLoader::new_from_str(
        r#"
instance_name = "quicklan-relay-lab"
dhcp = false
listeners = ["tcp://0.0.0.0:11010", "udp://0.0.0.0:11010"]
routes = []
stun_servers = []
stun_servers_v6 = []
exit_nodes = []
[network_identity]
network_name = "quicklan-isolated-lab-relay"
network_secret = "separate-non-member-relay"
[flags]
no_tun = true
enable_encryption = true
enable_ipv6 = false
disable_upnp = true
accept_dns = false
enable_exit_node = false
proxy_forward_by_system = false
relay_network_whitelist = "*"
disable_relay_data = false
disable_kcp_input = true
disable_quic_input = true
disable_p2p = true
disable_tcp_hole_punching = true
disable_udp_hole_punching = true
"#,
    )
    .unwrap();
    cfg.set_secure_mode(Some(
        process_secure_mode_cfg(SecureModeConfig {
            enabled: true,
            local_private_key: None,
            local_public_key: None,
        })
        .unwrap(),
    ));
    let mut instance = Instance::new(cfg);
    instance.run().await.unwrap();
    println!("READY");
    tokio::task::spawn_blocking(|| {
        use std::io::Read;
        let _ = std::io::stdin().read(&mut [0u8]);
    })
    .await
    .unwrap();
    instance.clear_resources().await;
}
