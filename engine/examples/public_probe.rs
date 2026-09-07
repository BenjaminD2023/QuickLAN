//! Explicit, bounded, unprivileged compatibility check. No adapters or system routes.
use easytier::{
    common::{
        config::{ConfigLoader, TomlConfigLoader, process_secure_mode_cfg},
        stun::StunInfoCollector,
    },
    instance::instance::Instance,
    proto::common::SecureModeConfig,
};
use easytier::{
    peers::PeerPacketFilter,
    tunnel::packet_def::{PacketType, ZCPacket},
};
use quicklan_core::{
    adapter::candidate_config,
    model::{Bootstrap, Network, Policy, Secret, random_hex},
};
use std::{sync::Arc, time::Duration};
struct Capture(tokio::sync::mpsc::Sender<Vec<u8>>);
impl PeerPacketFilter for Capture {
    fn try_process_packet_from_peer<'a, 'f>(
        &'a self,
        packet: ZCPacket,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ZCPacket>> + Send + 'f>>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            if packet
                .peer_manager_header()
                .is_some_and(|h| h.packet_type == PacketType::Data as u8)
            {
                let _ = self.0.send(packet.payload().to_vec()).await;
                None
            } else {
                Some(packet)
            }
        })
    }
}
fn packet(source: u8, destination: u8) -> Vec<u8> {
    let mut data = vec![0u8; 44];
    data[0] = 0x45;
    data[2..4].copy_from_slice(&44u16.to_be_bytes());
    data[8] = 64;
    data[9] = 17;
    data[12..16].copy_from_slice(&[10, 253, 249, source]);
    data[16..20].copy_from_slice(&[10, 253, 249, destination]);
    data[20..22].copy_from_slice(&41000u16.to_be_bytes());
    data[22..24].copy_from_slice(&41000u16.to_be_bytes());
    data[24..26].copy_from_slice(&24u16.to_be_bytes());
    data[28..].copy_from_slice(b"QuickLAN probe!!");
    let mut sum: u32 = data[..20]
        .chunks_exact(2)
        .map(|p| u16::from_be_bytes([p[0], p[1]]) as u32)
        .sum();
    while sum > 65535 {
        sum = (sum & 65535) + (sum >> 16);
    }
    data[10..12].copy_from_slice(&(!(sum as u16)).to_be_bytes());
    data
}
#[tokio::main]
async fn main() {
    let mut endpoint = std::env::args()
        .nth(1)
        .expect("explicit operator-permitted endpoint or --local required");
    let mut relay = None;
    if endpoint == "--local" {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        endpoint = format!("tcp://{}", listener.local_addr().unwrap());
        drop(listener);
        let cfg = TomlConfigLoader::new_from_str(&format!(
            r#"
listeners = ["{endpoint}"]
stun_servers = []
stun_servers_v6 = []
[network_identity]
network_name = "quicklan-probe-nonmember-relay"
network_secret = "independent-lab-relay"
[flags]
no_tun = true
enable_ipv6 = false
disable_upnp = true
disable_p2p = true
disable_tcp_hole_punching = true
disable_udp_hole_punching = true
relay_network_whitelist = "*"
"#
        ))
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
        instance
            .get_global_ctx()
            .replace_stun_info_collector(Box::new(StunInfoCollector::new(vec![], vec![], vec![])));
        instance.run().await.unwrap();
        relay = Some(instance);
    }
    let network = Network {
        id: random_hex(16).unwrap(),
        label: "QuickLAN compatibility test".into(),
        subnet: "10.253.249.0/24".into(),
        policy: Policy::Assisted,
        bootstrap: vec![Bootstrap {
            endpoint: if relay.is_some() {
                "tcp://192.0.2.1:11010".into()
            } else {
                endpoint.clone()
            },
            operator: "Explicit test target".into(),
        }],
    };
    let secret = Secret::generate().unwrap();
    let mut instances = vec![];
    let mut receivers = vec![];
    for n in 1..=2 {
        let cfg = TomlConfigLoader::new_from_str(
            &candidate_config(&network, &secret, "QuickLAN compatibility test").unwrap(),
        )
        .unwrap();
        // Loopback is intentionally forbidden in user invitations; substitute it
        // only in this unprivileged self-test after validating the normal config.
        if relay.is_some() {
            let mut peers = cfg.get_peers();
            peers[0].uri = endpoint.parse().unwrap();
            cfg.set_peers(peers);
        }
        cfg.set_dhcp(false);
        cfg.set_ipv4(Some(format!("10.253.249.{n}/24").parse().unwrap()));
        cfg.set_listeners(vec![]);
        let mut flags = cfg.get_flags();
        flags.no_tun = true;
        flags.disable_p2p = true;
        cfg.set_flags(flags);
        cfg.set_secure_mode(Some(
            process_secure_mode_cfg(SecureModeConfig {
                enabled: true,
                local_private_key: None,
                local_public_key: None,
            })
            .unwrap(),
        ));
        let mut instance = Instance::new(cfg);
        instance
            .get_global_ctx()
            .replace_stun_info_collector(Box::new(StunInfoCollector::new(vec![], vec![], vec![])));
        tokio::time::timeout(Duration::from_secs(15), instance.run())
            .await
            .unwrap()
            .unwrap();
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        instance
            .get_peer_manager()
            .add_packet_process_pipeline(Box::new(Arc::new(Capture(sender))))
            .await;
        receivers.push(receiver);
        instances.push(instance);
    }
    let ready = tokio::time::timeout(Duration::from_secs(45), async {
        loop {
            let a = instances[0].get_peer_manager();
            let b = instances[1].get_peer_manager();
            if a.list_routes()
                .await
                .iter()
                .any(|r| r.peer_id == instances[1].peer_id())
                && b.list_routes()
                    .await
                    .iter()
                    .any(|r| r.peer_id == instances[0].peer_id())
            {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .is_ok();
    let mut exchanged = ready;
    if ready {
        for (from, to) in [(0usize, 1usize), (1, 0)] {
            let payload = packet(from as u8 + 1, to as u8 + 1);
            let result = tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    instances[from]
                        .get_peer_manager()
                        .send_msg_by_ip(
                            ZCPacket::new_with_payload(&payload),
                            format!("10.253.249.{}", to + 1).parse().unwrap(),
                            true,
                        )
                        .await
                        .unwrap();
                    if let Ok(Some(received)) =
                        tokio::time::timeout(Duration::from_millis(500), receivers[to].recv()).await
                    {
                        assert_eq!(received, payload);
                        break;
                    }
                }
            })
            .await;
            exchanged &= result.is_ok();
        }
    }
    println!("Bidirectional encrypted application packets: {exchanged}");
    for instance in &mut instances {
        println!(
            "Secure relay connections: {}; discovered friend: {ready}",
            instance
                .get_peer_manager()
                .get_foreign_network_client()
                .list_public_peers()
                .await
                .len()
        );
        instance.clear_resources().await;
    }
    if let Some(mut relay) = relay {
        relay.clear_resources().await;
    }
    if !exchanged {
        std::process::exit(1);
    }
}
