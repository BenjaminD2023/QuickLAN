use crate::{
    error::{Error, Result},
    model::{Network, Policy, Secret},
    PROTOCOL,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const PREFIX: &str = "quicklan1:";
pub const MAX_TOKEN_BYTES: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invitation {
    pub version: u8,
    pub protocol: String,
    pub network: Network,
    pub credential: Secret,
}
impl Invitation {
    pub fn new(network: Network, credential: Secret) -> Self {
        Self {
            version: 1,
            protocol: PROTOCOL.into(),
            network,
            credential,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.protocol != PROTOCOL {
            return Err(Error::UnsupportedVersion);
        }
        self.network.validate()?;
        self.credential.validate()
    }
    pub fn encode(&self) -> Result<Zeroizing<String>> {
        self.validate()?;
        let json = Zeroizing::new(serde_json::to_vec(self).map_err(|_| Error::InvalidInvitation)?);
        let token = Zeroizing::new(format!("{PREFIX}{}", URL_SAFE_NO_PAD.encode(&*json)));
        if token.len() > MAX_TOKEN_BYTES {
            return Err(Error::InvalidInvitation);
        }
        Ok(token)
    }
    pub fn parse(input: &str) -> Result<Self> {
        if input.len() > MAX_TOKEN_BYTES {
            return Err(Error::InvalidInvitation);
        }
        let input = input.trim();
        let payload = input
            .strip_prefix(PREFIX)
            .ok_or(Error::UnsupportedVersion)?;
        let bytes = Zeroizing::new(
            URL_SAFE_NO_PAD
                .decode(payload)
                .map_err(|_| Error::InvalidInvitation)?,
        );
        let invitation: Self =
            serde_json::from_slice(&bytes).map_err(|_| Error::InvalidInvitation)?;
        invitation.validate()?;
        Ok(invitation)
    }
}

pub fn validate_endpoint(input: &str, policy: Policy) -> Result<()> {
    if input.len() > 256
        || !input.is_ascii()
        || input.contains(['\\', '%'])
        || input
            .bytes()
            .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
    {
        return Err(Error::InvalidEndpoint);
    }
    let url = url::Url::parse(input).map_err(|_| Error::InvalidEndpoint)?;
    if !matches!(url.scheme(), "tcp" | "udp")
        || !url.username().is_empty()
        || url.password().is_some()
        || !url.path().is_empty()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.port().is_none_or(|p| p == 0)
    {
        return Err(Error::InvalidEndpoint);
    }
    let raw_host = url
        .host_str()
        .ok_or(Error::InvalidEndpoint)?
        .trim_start_matches('[')
        .trim_end_matches(']');
    let host = match raw_host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => url::Host::Ipv4(ip),
        Ok(std::net::IpAddr::V6(ip)) => url::Host::Ipv6(ip),
        Err(_) => url::Host::Domain(raw_host),
    };
    match host {
        url::Host::Ipv4(ip) => {
            if ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || ip.is_loopback()
                || ip.is_link_local()
            {
                return Err(Error::InvalidEndpoint);
            }
            // Refuse alternative decimal/octal/hex IP spellings and URL normalization.
            if url.as_str() != input {
                return Err(Error::InvalidEndpoint);
            }
        }
        url::Host::Ipv6(ip) => {
            if ip.to_ipv4_mapped().is_some()
                || ip.is_unspecified()
                || ip.is_loopback()
                || ip.is_multicast()
                || ip.is_unicast_link_local()
            {
                return Err(Error::InvalidEndpoint);
            }
        }
        url::Host::Domain(host) => {
            if policy == Policy::Manual
                || host.len() > 253
                || !host.contains('.')
                || host.ends_with('.')
                || host.split('.').any(|label| {
                    label.is_empty()
                        || label.len() > 63
                        || label.starts_with('-')
                        || label.ends_with('-')
                        || !label
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                })
            {
                return Err(Error::InvalidEndpoint);
            }
        }
    }
    Ok(())
}
