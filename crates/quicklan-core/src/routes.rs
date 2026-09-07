use crate::error::{Error, Result};
use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub fn validate_subnet(value: &str) -> Result<Ipv4Net> {
    let subnet: Ipv4Net = value.parse().map_err(|_| Error::InvalidSubnet)?;
    if subnet.prefix_len() != 24
        || subnet.addr() != subnet.network()
        || subnet.to_string() != value
        || !subnet.addr().is_private()
    {
        return Err(Error::InvalidSubnet);
    }
    Ok(subnet)
}
pub fn overlaps(a: Ipv4Net, b: Ipv4Net) -> bool {
    a.contains(&b.network()) || b.contains(&a.network())
}

/// Only routes wholly inside private address space compete with our private
/// overlay. Internet VPN capture routes (0/1, 128/1, 8/5, etc.) are less-specific
/// catch-all routes, not LAN allocations. The overlay /24 takes precedence;
/// every existing route is left in place. A private VPN route such as 10/8 is
/// still a conflict, as are LAN subnets and host routes.
pub fn check_conflicts(subnet: &str, routes: &[Ipv4Net], saved: &[String]) -> Result<()> {
    let subnet = validate_subnet(subnet)?;
    for route in routes {
        if route.network().is_private()
            && route.broadcast().is_private()
            && overlaps(subnet, *route)
        {
            return Err(Error::RouteConflict);
        }
    }
    for value in saved {
        if overlaps(subnet, validate_subnet(value)?) {
            return Err(Error::RouteConflict);
        }
    }
    Ok(())
}

/// Control-boundary validation supplements (and never substitutes for) core packet enforcement.
pub fn validate_advertisement(
    subnet: &str,
    virtual_ip: Ipv4Addr,
    learned_subnets: &[String],
) -> Result<()> {
    let net = validate_subnet(subnet)?;
    if !learned_subnets.is_empty()
        || !net.contains(&virtual_ip)
        || virtual_ip == net.network()
        || virtual_ip == net.broadcast()
    {
        return Err(Error::RouteConflict);
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedResources {
    pub session_id: String,
    pub interface_id: Option<String>,
    pub routes: Vec<String>,
}
impl OwnedResources {
    pub fn validate(&self, subnet: &str) -> Result<()> {
        if !crate::model::valid_hex(&self.session_id, 32) || self.routes.iter().any(|r| r != subnet)
        {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}

/// Choose an unused private /24 without creating adapters or contacting a server.
pub fn choose_subnet(routes: &[Ipv4Net], saved: &[String]) -> Result<String> {
    use rand::Rng;
    let pools: [Ipv4Net; 3] = ["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"]
        .map(|s| s.parse().expect("constant subnet"));
    for pool in pools {
        let count = 1u32 << (24 - pool.prefix_len());
        let start = rand::thread_rng().gen_range(0..count);
        for offset in 0..count {
            let addr = Ipv4Addr::from(u32::from(pool.network()) + ((start + offset) % count) * 256);
            let candidate = format!("{addr}/24");
            if check_conflicts(&candidate, routes, saved).is_ok() {
                return Ok(candidate);
            }
        }
    }
    Err(Error::RouteConflict)
}
