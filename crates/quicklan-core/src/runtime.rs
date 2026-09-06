//! Application boundary to a real, separately owned native networking process.
use crate::{
    adapter::Peer,
    error::{Error, Result},
    helper::HelperRequest,
};

pub enum RuntimeEvent {
    Joining,
    State {
        virtual_ip: Option<String>,
        peers: Vec<Peer>,
    },
    Stopped,
    Failed(Error),
}

pub trait NetworkRuntime: Send {
    fn available(&self) -> bool;
    /// Starts a worker; never blocks the application's mutex on an OS prompt.
    fn start(&mut self, request: HelperRequest) -> Result<()>;
    /// True means wait for the real process's shutdown acknowledgement.
    fn stop(&mut self) -> Result<bool>;
    fn poll(&mut self) -> Vec<RuntimeEvent>;
}
