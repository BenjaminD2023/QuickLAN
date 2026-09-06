use crate::{
    error::Error,
    lifecycle::{Phase, Snapshot},
    CORE_VERSION,
};
use serde::Serialize;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub sequence: u64,
    pub phase: Phase,
    pub code: Option<Error>,
}
#[derive(Default)]
pub struct EventLog {
    events: VecDeque<Event>,
    sequence: u64,
}
impl EventLog {
    pub fn record(&mut self, state: &Snapshot) {
        self.sequence = self.sequence.saturating_add(1);
        if self.events.len() == 100 {
            self.events.pop_front();
        }
        self.events.push_back(Event {
            sequence: self.sequence,
            phase: state.phase,
            code: state.error.clone(),
        });
    }
    pub fn export(&self, state: &Snapshot) -> DiagnosticReport {
        DiagnosticReport { product:crate::PRODUCT_NAME,app_version:env!("CARGO_PKG_VERSION"),core_pin:CORE_VERSION,
            platform:std::env::consts::OS,architecture:std::env::consts::ARCH,phase:state.phase,
            peer_count:state.peers.len(),events:self.events.iter().cloned().collect(),
            exclusion_notice:"No invitations, credentials, device/network names, IP addresses, payloads or personal file paths. Nothing is uploaded." }
    }
}
#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    pub product: &'static str,
    pub app_version: &'static str,
    pub core_pin: &'static str,
    pub platform: &'static str,
    pub architecture: &'static str,
    pub phase: Phase,
    pub peer_count: usize,
    pub events: Vec<Event>,
    pub exclusion_notice: &'static str,
}
