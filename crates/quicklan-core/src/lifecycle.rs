use crate::{
    adapter::Peer,
    error::{Error, Result},
    model::valid_hex,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Disconnected,
    Starting,
    Joining,
    Connected,
    Reconnecting,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Snapshot {
    pub phase: Phase,
    pub network_id: Option<String>,
    pub virtual_ip: Option<String>,
    pub peers: Vec<Peer>,
    pub error: Option<Error>,
}

/// The caller holds the application mutex through every operation. Generation
/// numbers invalidate late core replies after stop/restart, including same network.
#[derive(Debug, Default)]
pub struct Lifecycle {
    generation: u64,
    state: Snapshot,
}
impl Lifecycle {
    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }
    pub fn begin_start(&mut self, id: &str) -> Result<u64> {
        if !valid_hex(id, 32) {
            return Err(Error::InvalidInvitation);
        }
        if !matches!(self.state.phase, Phase::Disconnected | Phase::Failed) {
            return Err(Error::Busy);
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::InvalidTransition)?;
        self.state = Snapshot {
            phase: Phase::Starting,
            network_id: Some(id.into()),
            ..Default::default()
        };
        Ok(self.generation)
    }
    pub fn transition(&mut self, generation: u64, next: Phase) -> Result<()> {
        use Phase::*;
        if generation != self.generation
            || !matches!(
                (self.state.phase, next),
                (Starting, Joining)
                    | (Joining, Connected)
                    | (Connected, Reconnecting)
                    | (Reconnecting, Connected)
            )
        {
            return Err(Error::InvalidTransition);
        }
        self.state.phase = next;
        Ok(())
    }
    pub fn observe(
        &mut self,
        generation: u64,
        virtual_ip: Option<String>,
        peers: Vec<Peer>,
    ) -> Result<()> {
        if generation != self.generation
            || !matches!(
                self.state.phase,
                Phase::Joining | Phase::Connected | Phase::Reconnecting
            )
        {
            return Err(Error::InvalidTransition);
        }
        self.state.virtual_ip = virtual_ip;
        self.state.peers = peers;
        Ok(())
    }
    pub fn fail(&mut self, generation: u64, error: Error) -> Result<()> {
        if generation != self.generation || self.state.phase == Phase::Disconnected {
            return Err(Error::InvalidTransition);
        }
        self.state.phase = Phase::Failed;
        self.state.peers.clear();
        self.state.virtual_ip = None;
        self.state.error = Some(error);
        Ok(())
    }
    pub fn begin_stop(&mut self) -> Result<u64> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::InvalidTransition)?;
        self.state.phase = Phase::Stopping;
        self.state.peers.clear();
        Ok(self.generation)
    }
    pub fn finish_stop(&mut self, generation: u64) -> Result<()> {
        if generation != self.generation || self.state.phase != Phase::Stopping {
            return Err(Error::InvalidTransition);
        }
        self.state = Snapshot::default();
        Ok(())
    }
    pub fn is_active(&self, id: &str) -> bool {
        self.state.network_id.as_deref() == Some(id)
            && !matches!(self.state.phase, Phase::Disconnected | Phase::Failed)
    }
}
