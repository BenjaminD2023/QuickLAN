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
#[cfg(not(target_os = "android"))]
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use quicklan_core::{
    adapter::{candidate_config, Peer, PeerPath},
    error::{Error, Result},
    model::{Network, Secret},
    protocol::EngineReply,
};
use std::{net::Ipv4Addr, time::Duration};

#[cfg(target_os = "android")]
pub trait MobileTun: Send + Sync {
    fn establish(&self, ip: &str, subnet: &str) -> Result<i32>;
    fn close(&self);
}

#[cfg(target_os = "android")]
struct MobileAttach {
    provider: std::sync::Arc<dyn MobileTun>,
    fd: Option<std::os::fd::OwnedFd>,
    attached_ip: Option<Ipv4Addr>,
}

pub struct NetworkSession {
    instance: Instance,
    subnet: String,
    direct_only: bool,
    #[cfg(target_os = "android")]
    mobile: Option<MobileAttach>,
}
impl NetworkSession {
    fn prepare_instance(
        network: &Network,
        credential: &Secret,
        nickname: &str,
        session_id: &str,
    ) -> Result<Instance> {
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
        Ok(instance)
    }

    async fn wait_for_dhcp(&mut self) -> Result<()> {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                if self.assigned_ip().is_some() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| Error::CoreFailed)
    }

    fn assigned_ip(&self) -> Option<Ipv4Addr> {
        self.instance.get_global_ctx().get_ipv4().and_then(|ip| {
            let address = ip.address();
            quicklan_core::routes::validate_advertisement(&self.subnet, address, &[])
                .ok()
                .map(|_| address)
        })
    }

    #[cfg(not(target_os = "android"))]
    pub async fn start(
        network: &Network,
        credential: &Secret,
        nickname: &str,
        session_id: &str,
    ) -> Result<Self> {
        quicklan_core::routes::check_conflicts(
            &network.subnet,
            &quicklan_core::system_routes::read()?,
            &[],
        )?;
        let instance = Self::prepare_instance(network, credential, nickname, session_id)?;
        let mut session = Self {
            instance,
            subnet: network.subnet.clone(),
            direct_only: network.policy == quicklan_core::model::Policy::DirectOnly,
        };
        match tokio::time::timeout(Duration::from_secs(30), session.instance.run()).await {
            Ok(Ok(())) => {
                // Upstream DHCP creates the virtual interface asynchronously.
                // Do not report an address until creation has succeeded.
                match session.wait_for_dhcp().await {
                    Ok(()) => Ok(session),
                    Err(error) => {
                        session.stop().await;
                        Err(error)
                    }
                }
            }
            _ => {
                session.stop().await;
                Err(Error::CoreFailed)
            }
        }
    }

    #[cfg(target_os = "android")]
    pub async fn start_mobile(
        network: &Network,
        credential: &Secret,
        nickname: &str,
        session_id: &str,
        tun: std::sync::Arc<dyn MobileTun>,
        stop: &std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        reject_overlay_bootstrap(network)?;
        let instance = Self::prepare_instance(network, credential, nickname, session_id)?;
        let mut session = Self {
            instance,
            subnet: network.subnet.clone(),
            direct_only: network.policy == quicklan_core::model::Policy::DirectOnly,
            mobile: Some(MobileAttach {
                provider: tun,
                fd: None,
                attached_ip: None,
            }),
        };
        let started = tokio::select! {
            result = async {
                tokio::time::timeout(Duration::from_secs(30), session.instance.run())
                    .await
                    .map_err(|_| Error::CoreFailed)?
                    .map_err(|_| Error::CoreFailed)?;
                session.wait_for_dhcp().await
            } => result,
            _ = async {
                while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            } => Err(Error::CoreFailed),
        };
        if let Err(error) = started {
            session.stop().await;
            return Err(error);
        }
        let Some(ip) = session.assigned_ip() else {
            session.stop().await;
            return Err(Error::CoreFailed);
        };
        if let Err(error) = session.ensure_mobile_tun(ip).await {
            session.stop().await;
            return Err(error);
        }
        if stop.load(std::sync::atomic::Ordering::SeqCst) {
            session.stop().await;
            return Err(Error::CoreFailed);
        }
        Ok(session)
    }

    pub async fn state(&mut self) -> Result<EngineReply> {
        let own_ip = self
            .instance
            .get_global_ctx()
            .get_ipv4()
            .map(|ip| ip.address());
        if let Some(ip) = own_ip {
            quicklan_core::routes::validate_advertisement(&self.subnet, ip, &[])?;
            #[cfg(target_os = "android")]
            self.ensure_mobile_tun(ip).await?;
            #[cfg(not(target_os = "android"))]
            {
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
        } else {
            #[cfg(target_os = "android")]
            self.release_mobile_tun().await;
        }
        let manager = self.instance.get_peer_manager();
        let map = manager.get_peer_map();
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
        #[cfg(target_os = "android")]
        self.release_mobile_tun().await;
        let _ = tokio::time::timeout(Duration::from_secs(5), self.instance.clear_resources()).await;
    }

    #[cfg(target_os = "android")]
    async fn ensure_mobile_tun(&mut self, ip: Ipv4Addr) -> Result<()> {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        if self.mobile.as_ref().is_some_and(|tun| {
            tun.attached_ip == Some(ip) && tun.fd.as_ref().is_some_and(|fd| fd.as_raw_fd() >= 0)
        }) {
            return Ok(());
        }
        self.release_mobile_tun().await;
        let provider = self
            .mobile
            .as_ref()
            .ok_or(Error::CoreFailed)?
            .provider
            .clone();
        let fd = provider.establish(&ip.to_string(), &self.subnet)?;
        if fd < 0 {
            provider.close();
            return Err(Error::CoreFailed);
        }
        let owned = unsafe { OwnedFd::from_raw_fd(fd) };
        let raw = owned.as_raw_fd();
        match Instance::setup_nic_ctx_for_mobile(
            self.instance.get_nic_ctx(),
            self.instance.get_global_ctx(),
            self.instance.get_peer_manager(),
            self.instance.get_peer_packet_receiver(),
            raw,
        )
        .await
        {
            Ok(()) => {
                if let Some(tun) = self.mobile.as_mut() {
                    tun.fd = Some(owned);
                    tun.attached_ip = Some(ip);
                    Ok(())
                } else {
                    drop(owned);
                    provider.close();
                    Err(Error::CoreFailed)
                }
            }
            Err(_) => {
                drop(owned);
                provider.close();
                Err(Error::CoreFailed)
            }
        }
    }

    #[cfg(target_os = "android")]
    async fn release_mobile_tun(&mut self) {
        if !self
            .mobile
            .as_ref()
            .is_some_and(|tun| tun.fd.is_some() || tun.attached_ip.is_some())
        {
            return;
        }
        let _ = Instance::setup_nic_ctx_for_mobile(
            self.instance.get_nic_ctx(),
            self.instance.get_global_ctx(),
            self.instance.get_peer_manager(),
            self.instance.get_peer_packet_receiver(),
            0,
        )
        .await;
        self.close_owned_tun();
    }

    #[cfg(target_os = "android")]
    fn close_owned_tun(&mut self) {
        if let Some(tun) = self.mobile.as_mut() {
            // Nic ctx already dropped the tun crate device with close_fd_on_drop(false).
            tun.fd.take();
            tun.attached_ip = None;
            tun.provider.close();
        }
    }
}

#[cfg(target_os = "android")]
fn reject_overlay_bootstrap(network: &Network) -> Result<()> {
    let subnet = quicklan_core::routes::validate_subnet(&network.subnet)?;
    for node in &network.bootstrap {
        if let Some(ip) = bootstrap_ipv4(&node.endpoint) {
            if subnet.contains(&ip) {
                return Err(Error::RouteConflict);
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "android")]
fn bootstrap_ipv4(endpoint: &str) -> Option<Ipv4Addr> {
    let rest = endpoint
        .strip_prefix("tcp://")
        .or_else(|| endpoint.strip_prefix("udp://"))?;
    let host = rest.rsplit_once(':')?.0;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    host.parse().ok()
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
