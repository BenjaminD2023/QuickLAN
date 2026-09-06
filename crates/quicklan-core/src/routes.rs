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

/// A default route alone is not a collision; more-specific VPN split defaults are.
pub fn check_conflicts(subnet: &str, routes: &[Ipv4Net], saved: &[String]) -> Result<()> {
    let subnet = validate_subnet(subnet)?;
    for route in routes {
        if route.prefix_len() != 0 && overlaps(subnet, *route) {
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
