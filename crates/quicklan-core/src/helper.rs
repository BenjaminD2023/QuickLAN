//! Fail-closed platform boundary. No unauthenticated privileged listener exists.
//! OS installers / authenticated core management are release gates, not simulated.
use crate::{
    error::{Error, Result},
    model::{valid_hex, validate_label, Network, Secret},
};
use serde::{Deserialize, Serialize};

pub const MAX_REQUEST_BYTES: usize = 16_384;
#[derive(Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum HelperRequest {
    Start {
        protocol: u8,
        session_id: String,
        network: Network,
        credential: Secret,
        nickname: String,
    },
    Stop {
        protocol: u8,
        session_id: String,
    },
    Status {
        protocol: u8,
    },
}
impl HelperRequest {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(Error::Unauthorized);
        }
        let request: Self = serde_json::from_slice(bytes).map_err(|_| Error::Unauthorized)?;
        match &request {
            Self::Start {
                protocol,
                session_id,
                network,
                credential,
                nickname,
            } => {
                if *protocol != 1 || !valid_hex(session_id, 32) {
                    return Err(Error::Unauthorized);
                }
                network.validate()?;
                credential.validate()?;
                validate_label(nickname)?;
            }
            Self::Stop {
                protocol,
                session_id,
            } => {
                if *protocol != 1 || !valid_hex(session_id, 32) {
                    return Err(Error::Unauthorized);
                }
            }
            Self::Status { protocol } => {
                if *protocol != 1 {
                    return Err(Error::Unauthorized);
                }
            }
        }
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HelperStatus {
    pub installed: bool,
    pub connection_enabled: bool,
    pub code: Error,
    pub release_gaps: Vec<&'static str>,
}
pub fn status() -> HelperStatus {
    HelperStatus { installed:false, connection_enabled:false, code:Error::HelperUnavailable,
        release_gaps:vec!["Authenticated core management and signed OS helper installation are not implemented.","Stock core retains implicit TCP STUN servers.","Direct-only path migration and hostile learned-route enforcement are not verified.","Windows and both macOS architectures require real virtual-IP and install/uninstall tests."] }
}
pub fn connect(_request: HelperRequest) -> Result<()> {
    Err(Error::HelperUnavailable)
}
pub fn repair() -> Result<()> {
    Err(Error::ReleaseGate)
}
