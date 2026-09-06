//! Version-pinned EasyTier adapter. No production process launcher is exposed.
use crate::{
    error::{Error, Result},
    model::{Network, Policy, Secret},
    CORE_VERSION,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::Ipv4Addr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerPath {
    Local,
    Direct,
    Relayed,
    Unreachable,
    Unknown,
}
#[derive(Debug, Clone, Serialize)]
pub struct Peer {
    pub id: String,
    pub nickname: String,
    pub virtual_ip: Option<String>,
    pub path: PeerPath,
    pub latency_ms: Option<f64>,
    pub identity_verified: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PeerRow {
    cidr: String,
    ipv4: String,
    hostname: String,
    cost: String,
    lat_ms: String,
    loss_rate: String,
    rx_bytes: String,
    tx_bytes: String,
    tunnel_proto: String,
    nat_type: String,
    id: String,
    version: String,
}

pub fn parse_peers(input: &[u8], version: &str, subnet: &str) -> Result<Vec<Peer>> {
    if version != CORE_VERSION {
        return Err(Error::IncompatibleCore);
    }
    if input.len() > 1_048_576 {
        return Err(Error::UnsupportedCoreOutput);
    }
    let rows: Vec<PeerRow> =
        serde_json::from_slice(input).map_err(|_| Error::UnsupportedCoreOutput)?;
    if rows.len() > 4096 {
        return Err(Error::UnsupportedCoreOutput);
    }
    let mut ids = std::collections::HashSet::new();
    rows.into_iter()
        .map(|row| {
            if row.id.parse::<u32>().is_err()
                || !ids.insert(row.id.clone())
                || row.hostname.len() > 256
            {
                return Err(Error::UnsupportedCoreOutput);
            }
            let path = match row.cost.as_str() {
                "Local" => PeerPath::Local,
                "p2p" => PeerPath::Direct,
                // Cost strings in this release are reviewed below; unknown strings fail closed.
                s if s.starts_with("relay(")
                    && s.ends_with(')')
                    && s[6..s.len() - 1].parse::<u32>().is_ok() =>
                {
                    PeerPath::Relayed
                }
                _ => return Err(Error::UnsupportedCoreOutput),
            };
            let virtual_ip = if row.ipv4.is_empty() {
                None
            } else {
                let ip: Ipv4Addr = row.ipv4.parse().map_err(|_| Error::UnsupportedCoreOutput)?;
                crate::routes::validate_advertisement(subnet, ip, &[])?;
                if row.cidr != format!("{ip}/24") {
                    return Err(Error::UnsupportedCoreOutput);
                }
                Some(ip.to_string())
            };
            // Upstream table conversion uses 0 as a fallback for missing measurements.
            // Relayed lat_ms is a path estimate, not a measured end-to-end RTT.
            let latency_ms = if path == PeerPath::Direct && row.lat_ms != "-" {
                let latency = row
                    .lat_ms
                    .parse::<f64>()
                    .map_err(|_| Error::UnsupportedCoreOutput)?;
                if !latency.is_finite() || latency < 0.0 {
                    return Err(Error::UnsupportedCoreOutput);
                }
                (latency > 0.0).then_some(latency)
            } else {
                None
            };
            // These fields are schema-checked, but not presented as trustworthy measurements.
            let _unused = (
                row.loss_rate,
                row.rx_bytes,
                row.tx_bytes,
                row.tunnel_proto,
                row.nat_type,
                row.version,
            );
            let nickname = row
                .hostname
                .chars()
                .filter(|c| {
                    !c.is_control()
                        && !matches!(c,'\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
                })
                .take(64)
                .collect();
            Ok(Peer {
                id: row.id,
                nickname,
                virtual_ip,
                path,
                latency_ms,
                identity_verified: false,
            })
        })
        .collect()
}

pub fn verify_bytes(bytes: &[u8], expected_hex: &str) -> Result<()> {
    if format!("{:x}", Sha256::digest(bytes)) != expected_hex {
        return Err(Error::IncompatibleCore);
    }
    Ok(())
}

/// Render only a verified, bounded subset. This is a candidate configuration,
/// never permission to run an elevated stock core; see helper release gates.
pub fn candidate_config(
    network: &Network,
    secret: &Secret,
    nickname: &str,
) -> Result<zeroize::Zeroizing<String>> {
    network.validate()?;
    secret.validate()?;
    crate::model::validate_label(nickname)?;
    use toml::Value;
    let mut cfg = toml::map::Map::new();
    cfg.insert(
        "instance_name".into(),
        Value::String(format!("quicklan-{}", network.id)),
    );
    cfg.insert("hostname".into(), Value::String(nickname.into()));
    cfg.insert("ipv4".into(), Value::String(network.subnet.clone()));
    cfg.insert("dhcp".into(), Value::Boolean(true));
    for key in ["routes", "stun_servers", "stun_servers_v6", "exit_nodes"] {
        cfg.insert(key.into(), Value::Array(vec![]));
    }
    for key in ["ipv6_public_addr_auto", "ipv6_public_addr_provider"] {
        cfg.insert(key.into(), Value::Boolean(false));
    }
    cfg.insert(
        "listeners".into(),
        Value::Array(vec![
            Value::String("tcp://0.0.0.0:11010".into()),
            Value::String("udp://0.0.0.0:11010".into()),
        ]),
    );
    let mut identity = toml::map::Map::new();
    identity.insert("network_name".into(), Value::String(network.id.clone()));
    identity.insert(
        "network_secret".into(),
        Value::String(secret.expose().into()),
    );
    cfg.insert("network_identity".into(), Value::Table(identity));
    cfg.insert(
        "peer".into(),
        Value::Array(
            network
                .bootstrap
                .iter()
                .map(|node| {
                    let mut peer = toml::map::Map::new();
                    peer.insert("uri".into(), Value::String(node.endpoint.clone()));
                    Value::Table(peer)
                })
                .collect(),
        ),
    );
    let mut flags = toml::map::Map::new();
    for (key, value) in [
        ("enable_encryption", true),
        ("enable_ipv6", false),
        ("disable_upnp", true),
        ("disable_relay_data", true),
        ("disable_relay_kcp", true),
        ("disable_relay_quic", true),
        ("disable_kcp_input", true),
        ("disable_quic_input", true),
        ("accept_dns", false),
        ("enable_exit_node", false),
        ("proxy_forward_by_system", false),
        ("enable_udp_broadcast_relay", false),
        ("private_mode", network.policy == Policy::Manual),
        (
            "disable_tcp_hole_punching",
            network.policy == Policy::Manual,
        ),
        (
            "disable_udp_hole_punching",
            network.policy == Policy::Manual,
        ),
    ] {
        flags.insert(key.into(), Value::Boolean(value));
    }
    flags.insert(
        "relay_network_whitelist".into(),
        Value::String(String::new()),
    );
    flags.insert(
        "encryption_algorithm".into(),
        Value::String("aes-256-gcm".into()),
    );
    flags.insert("mtu".into(), Value::Integer(1360));
    cfg.insert("flags".into(), Value::Table(flags));
    let text =
        zeroize::Zeroizing::new(toml::to_string(&cfg).map_err(|_| Error::InvalidInvitation)?);
    // toml's intermediate owned strings are an unavoidable temporary plaintext copy.
    Ok(text)
}
