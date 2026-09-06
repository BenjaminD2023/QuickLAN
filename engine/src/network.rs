//! Owns exactly one upstream Instance; deliberately never starts EasyTier RPC.
use easytier::{
    common::{
        config::{ConfigLoader, TomlConfigLoader, process_secure_mode_cfg},
        stun::StunInfoCollector,
    },
    instance::instance::Instance,
    peers::route_trait::NextHopPolicy,
    proto::common::SecureModeConfig,
};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use quicklan_core::{
    adapter::{Peer, PeerPath, candidate_config},
    error::{Error, Result},
    model::{Network, Secret},
    protocol::EngineReply,
};
use std::{net::Ipv4Addr, time::Duration};

pub struct NetworkSession {
    instance: Instance,
    subnet: String,
    direct_only: bool,
}
impl NetworkSession {
    pub async fn start(
        network: &Network,
        credential: &Secret,
        nickname: &str,
        session_id: &str,
    ) -> Result<Self> {
        if !quicklan_core::model::valid_hex(session_id, 32) {
            return Err(Error::Unauthorized);
        }
        #[cfg(windows)]
        {
            let path = std::env::current_exe().map_err(|_| Error::IncompatibleCore)?;
            let dll = path
                .parent()
                .ok_or(Error::IncompatibleCore)?
                .join("wintun.dll");
            let bytes = std::fs::read(dll).map_err(|_| Error::IncompatibleCore)?;
            quicklan_core::adapter::verify_bytes(
                &bytes,
                "e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce",
            )?;
        }
        quicklan_core::routes::check_conflicts(
            &network.subnet,
            &quicklan_core::system_routes::read()?,
            &[],
        )?;
        let text = candidate_config(network, credential, nickname)?;
        let cfg = TomlConfigLoader::new_from_str(&text).map_err(|_| Error::CoreFailed)?;
        // Unique ownership avoids opening a shared adapter created by another VPN.
        let mut flags = cfg.get_flags();
        flags.dev_name = format!("ql{}", &session_id[..10]);
        cfg.set_flags(flags);
        cfg.set_secure_mode(Some(
            process_secure_mode_cfg(SecureModeConfig {
                enabled: true,
                local_private_key: None,
                local_public_key: None,
            })
            .map_err(|_| Error::CoreFailed)?,
        ));
        let instance = Instance::new(cfg);
        // Replace *all* discovery defaults before any networking tasks start.
        // Custom configured peer endpoints are the only network destinations seeded here.
        instance
            .get_global_ctx()
            .replace_stun_info_collector(Box::new(StunInfoCollector::new(vec![], vec![], vec![])));
        let mut session = Self {
            instance,
            subnet: network.subnet.clone(),
            direct_only: network.policy == quicklan_core::model::Policy::DirectOnly,
        };
        match tokio::time::timeout(Duration::from_secs(30), session.instance.run()).await {
            Ok(Ok(())) => {
                // Upstream DHCP creates the virtual interface asynchronously.
                // Do not report an address until creation has succeeded.
                let ready = tokio::time::timeout(Duration::from_secs(20), async {
                    loop {
                        if session
                            .instance
                            .get_global_ctx()
                            .get_ipv4()
                            .is_some_and(|ip| {
                                quicklan_core::routes::validate_advertisement(
                                    &session.subnet,
                                    ip.address(),
                                    &[],
                                )
                                .is_ok()
                            })
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                })
                .await;
                if ready.is_ok() {
                    Ok(session)
                } else {
                    session.stop().await;
                    Err(Error::CoreFailed)
                }
            }
            _ => {
                session.stop().await;
                Err(Error::CoreFailed)
            }
        }
    }
    pub async fn state(&self) -> Result<EngineReply> {
        let manager = self.instance.get_peer_manager();
        let map = manager.get_peer_map();
        let own_ip = self
            .instance
            .get_global_ctx()
            .get_ipv4()
            .map(|ip| ip.address());
        if let Some(ip) = own_ip {
            quicklan_core::routes::validate_advertisement(&self.subnet, ip, &[])?;
            let name = self
                .instance
                .quicklan_ifname()
                .await
                .ok_or(Error::CoreFailed)?;
            let interfaces = NetworkInterface::show().map_err(|_| Error::CoreFailed)?;
            if !interfaces.iter().any(|iface| {
                iface.name == name
                    && iface
                        .addr
                        .iter()
                        .any(|addr| addr.ip() == std::net::IpAddr::V4(ip))
            }) {
                return Err(Error::CoreFailed);
            }
        }
        let mut peers = Vec::new();
        for route in manager.list_routes().await {
            if route.peer_id == self.instance.peer_id() {
                continue;
            }
            if peers.len() >= 4096 {
                return Err(Error::UnsupportedCoreOutput);
            }
            let virtual_ip = match route.ipv4_addr {
                Some(address) => {
                    if address.network_length != 24 {
                        return Err(Error::RouteConflict);
                    }
                    let ip: Ipv4Addr = address.address.ok_or(Error::UnsupportedCoreOutput)?.into();
                    quicklan_core::routes::validate_advertisement(
                        &self.subnet,
                        ip,
                        &route.proxy_cidrs,
                    )?;
                    Some(ip.to_string())
                }
                None => None,
            };
            let live = map
                .list_peer_conns(route.peer_id)
                .await
                .unwrap_or_default()
                .into_iter()
                .filter(|c| !c.is_closed)
                .collect::<Vec<_>>();
            let direct = !live.is_empty();
            let latency_ms = live
                .iter()
                .filter_map(|c| c.stats.as_ref())
                .filter(|s| s.latency_us > 0)
                .map(|s| s.latency_us)
                .min()
                .map(|us| us as f64 / 1000.0);
            let gateway = map
                .get_gateway_peer_id(route.peer_id, NextHopPolicy::LeastHop)
                .await;
            let relay_live = if let Some(gateway) = gateway {
                map.list_peer_conns(gateway)
                    .await
                    .unwrap_or_default()
                    .iter()
                    .any(|conn| !conn.is_closed)
                    || manager.get_foreign_network_client().has_next_hop(gateway)
            } else {
                manager
                    .get_foreign_network_client()
                    .has_next_hop(route.peer_id)
            };
            let path = if direct {
                PeerPath::Direct
            } else if relay_live && !self.direct_only {
                PeerPath::Relayed
            } else {
                PeerPath::Unreachable
            };
            let nickname = route
                .hostname
                .chars()
                .filter(|c| {
                    !c.is_control()
                        && !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
                })
                .take(64)
                .collect();
            peers.push(Peer {
                id: route.peer_id.to_string(),
                nickname,
                virtual_ip,
                path,
                latency_ms,
                identity_verified: false,
            });
        }
        Ok(EngineReply::State {
            virtual_ip: own_ip.map(|ip| ip.to_string()),
            peers,
        })
    }
    pub async fn stop(&mut self) {
        let _ = tokio::time::timeout(Duration::from_secs(5), self.instance.clear_resources()).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn packet_scope_rejects_foreign_addresses_broadcast_ipv6_and_truncation() {
        use easytier::quicklan_policy::allow_ipv4;
        let proposal = Some("10.73.42.0/24".parse().unwrap());
        let local = Some("10.73.42.1/24".parse().unwrap());
        let mut packet = [0u8; 28];
        packet[0] = 0x45;
        packet[3] = 28;
        packet[12..16].copy_from_slice(&[10, 73, 42, 2]);
        packet[16..20].copy_from_slice(&[10, 73, 42, 1]);
        assert!(allow_ipv4(&packet, proposal, local, true));
        assert!(!allow_ipv4(&packet, proposal, local, false));
        for len in 0..packet.len() {
            assert!(!allow_ipv4(&packet[..len], proposal, local, true));
        }
        for source in [
            [8, 8, 8, 8],
            [10, 73, 43, 2],
            [10, 73, 42, 0],
            [10, 73, 42, 255],
        ] {
            packet[12..16].copy_from_slice(&source);
            assert!(!allow_ipv4(&packet, proposal, local, true));
        }
        packet[12..16].copy_from_slice(&[10, 73, 42, 2]);
        packet[0] = 0x65;
        assert!(!allow_ipv4(&packet, proposal, local, true));
        packet[0] = 0x4f;
        assert!(!allow_ipv4(&packet, proposal, local, true));
    }
}
