use crate::{
    diagnostics::{DiagnosticReport, EventLog},
    error::{Error, Result},
    helper,
    invitation::Invitation,
    lifecycle::{Lifecycle, Snapshot},
    model::{
        random_hex, validate_label, Bootstrap, Network, Policy, Preferences, SavedState, Secret,
    },
    runtime::{NetworkRuntime, RuntimeEvent},
    storage::Store,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JoinPreview {
    pub ticket: String,
    pub network: Network,
}
#[derive(Debug, Clone, Serialize)]
pub struct AppView {
    pub saved: SavedState,
    pub connection: Snapshot,
    pub helper: helper::HelperStatus,
}

pub struct App<S: Store> {
    store: S,
    saved: SavedState,
    lifecycle: Lifecycle,
    events: EventLog,
    pending: Option<(String, Invitation)>,
    runtime: Option<Box<dyn NetworkRuntime>>,
    active_generation: Option<u64>,
}
impl<S: Store> App<S> {
    pub fn new(store: S) -> Result<Self> {
        let saved = store.load()?;
        saved.validate()?;
        Ok(Self {
            store,
            saved,
            lifecycle: Lifecycle::default(),
            events: EventLog::default(),
            pending: None,
            runtime: None,
            active_generation: None,
        })
    }
    pub fn with_runtime(mut self, runtime: Box<dyn NetworkRuntime>) -> Self {
        self.runtime = Some(runtime);
        self
    }
    pub fn refresh(&mut self) -> Result<()> {
        let events = self.runtime.as_mut().map(|r| r.poll()).unwrap_or_default();
        let Some(generation) = self.active_generation else {
            return Ok(());
        };
        use crate::lifecycle::Phase;
        for event in events {
            let phase = self.lifecycle.snapshot().phase;
            match event {
                RuntimeEvent::Joining if phase == Phase::Starting => {
                    self.lifecycle.transition(generation, Phase::Joining)?
                }
                RuntimeEvent::State { virtual_ip, peers }
                    if matches!(
                        phase,
                        Phase::Joining | Phase::Connected | Phase::Reconnecting
                    ) =>
                {
                    let ready = virtual_ip.is_some();
                    self.lifecycle.observe(generation, virtual_ip, peers)?;
                    if ready && matches!(phase, Phase::Joining | Phase::Reconnecting) {
                        self.lifecycle.transition(generation, Phase::Connected)?;
                    } else if !ready && phase == Phase::Connected {
                        self.lifecycle.transition(generation, Phase::Reconnecting)?;
                    }
                }
                RuntimeEvent::Stopped if phase == Phase::Stopping => {
                    self.lifecycle.finish_stop(generation)?;
                    self.active_generation = None;
                }
                RuntimeEvent::Failed(error)
                    if !matches!(phase, Phase::Disconnected | Phase::Failed) =>
                {
                    self.lifecycle.fail(generation, error)?
                }
                _ => (), // Old observations cannot resurrect a stopped/failed session.
            }
            self.events.record(&self.lifecycle.snapshot());
        }
        Ok(())
    }
    pub fn view(&self) -> AppView {
        AppView {
            saved: self.saved.clone(),
            connection: self.lifecycle.snapshot(),
            helper: if self.runtime.as_ref().is_some_and(|r| r.available()) {
                helper::HelperStatus {
                    installed: true,
                    connection_enabled: true,
                    code: None,
                    release_gaps: vec![],
                }
            } else {
                helper::status()
            },
        }
    }
    pub fn service_endpoint(&mut self, peer_id: &str, port: u16) -> Result<std::net::SocketAddrV4> {
        self.refresh()?;
        let snapshot = self.lifecycle.snapshot();
        if port == 0 || snapshot.phase != crate::lifecycle::Phase::Connected {
            return Err(Error::InvalidService);
        }
        let peer = snapshot
            .peers
            .iter()
            .find(|peer| peer.id == peer_id)
            .ok_or(Error::InvalidService)?;
        let ip = peer
            .virtual_ip
            .as_ref()
            .ok_or(Error::InvalidService)?
            .parse()
            .map_err(|_| Error::InvalidService)?;
        let network = self.network(
            snapshot
                .network_id
                .as_deref()
                .ok_or(Error::InvalidService)?,
        )?;
        crate::routes::validate_advertisement(&network.subnet, ip, &[])?;
        Ok(std::net::SocketAddrV4::new(ip, port))
    }
    pub fn network(&self, id: &str) -> Result<&Network> {
        self.saved
            .networks
            .iter()
            .find(|n| n.id == id)
            .ok_or(Error::NotFound)
    }
    pub fn create(
        &mut self,
        label: String,
        subnet: String,
        policy: Policy,
        bootstrap: Vec<Bootstrap>,
        assistance_accepted: bool,
    ) -> Result<Network> {
        if policy == Policy::Assisted && !assistance_accepted {
            return Err(Error::AssistanceConsentRequired);
        }
        let network = Network {
            id: random_hex(16)?,
            label,
            subnet,
            policy,
            bootstrap,
        };
        network.validate()?;
        crate::routes::check_conflicts(
            &network.subnet,
            &[],
            &self
                .saved
                .networks
                .iter()
                .map(|n| n.subnet.clone())
                .collect::<Vec<_>>(),
        )?;
        self.insert(network.clone(), Secret::generate()?)?;
        Ok(network)
    }
    pub fn create_with_nickname(
        &mut self,
        label: String,
        subnet: String,
        policy: Policy,
        bootstrap: Vec<Bootstrap>,
        assistance_accepted: bool,
        nickname: String,
    ) -> Result<Network> {
        validate_label(&nickname)?;
        let previous = self.saved.preferences.nickname.clone();
        self.saved.preferences.nickname = nickname;
        let result = self.create(label, subnet, policy, bootstrap, assistance_accepted);
        if result.is_err() {
            self.saved.preferences.nickname = previous;
        }
        result
    }
    pub fn update_settings(
        &mut self,
        id: &str,
        policy: Policy,
        bootstrap: Vec<Bootstrap>,
        assistance_accepted: bool,
    ) -> Result<()> {
        if self.lifecycle.is_active(id) {
            return Err(Error::Busy);
        }
        if policy == Policy::Assisted && !assistance_accepted {
            return Err(Error::AssistanceConsentRequired);
        }
        let mut updated = self.saved.clone();
        let network = updated
            .networks
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(Error::NotFound)?;
        network.policy = policy;
        network.bootstrap = bootstrap;
        network.validate()?;
        self.store.save(&updated)?;
        self.saved = updated;
        Ok(())
    }
    fn insert(&mut self, network: Network, secret: Secret) -> Result<()> {
        if self.saved.networks.iter().any(|n| n.id == network.id) {
            return Err(Error::AlreadySaved);
        }
        let mut updated = self.saved.clone();
        updated.networks.push(network.clone());
        self.store.put_secret(&network.id, &secret)?;
        if let Err(error) = self.store.save(&updated) {
            // Crash between these stores can leave an inaccessible orphan credential,
            // but must never leave a plaintext secret or claim a saved network.
            let _ = self.store.remove_secret(&network.id);
            return Err(error);
        }
        self.saved = updated;
        Ok(())
    }
    pub fn preview(&mut self, token: &str) -> Result<JoinPreview> {
        self.pending = None;
        let invitation = Invitation::parse(token)?;
        let ticket = random_hex(16)?;
        let preview = JoinPreview {
            ticket: ticket.clone(),
            network: invitation.network.clone(),
        };
        self.pending = Some((ticket, invitation));
        Ok(preview)
    }
    pub fn cancel_preview(&mut self) {
        self.pending = None;
    }
    pub fn accept(
        &mut self,
        ticket: &str,
        trusted: bool,
        assistance_accepted: bool,
    ) -> Result<Network> {
        if !trusted {
            return Err(Error::ConfirmationRequired);
        }
        let (stored, invitation) = self.pending.as_ref().ok_or(Error::InvalidInvitation)?;
        if stored != ticket {
            return Err(Error::InvalidInvitation);
        }
        if invitation.network.policy == Policy::Assisted && !assistance_accepted {
            return Err(Error::AssistanceConsentRequired);
        }
        let network = invitation.network.clone();
        crate::routes::check_conflicts(
            &network.subnet,
            &[],
            &self
                .saved
                .networks
                .iter()
                .filter(|n| n.id != network.id)
                .map(|n| n.subnet.clone())
                .collect::<Vec<_>>(),
        )?;
        self.insert(network.clone(), invitation.credential.clone())?;
        self.pending = None;
        Ok(network)
    }
    pub fn invitation(&self, id: &str) -> Result<zeroize::Zeroizing<String>> {
        let network = self.network(id)?.clone();
        Invitation::new(network, self.store.get_secret(id)?).encode()
    }
    pub fn rename(&mut self, id: &str, label: String) -> Result<()> {
        validate_label(&label)?;
        let mut updated = self.saved.clone();
        updated
            .networks
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(Error::NotFound)?
            .label = label;
        self.store.save(&updated)?;
        self.saved = updated;
        Ok(())
    }
    pub fn forget(&mut self, id: &str) -> Result<()> {
        if self.lifecycle.is_active(id) {
            return Err(Error::Busy);
        }
        self.network(id)?;
        // Remove credential first; if metadata save fails, restore the credential.
        let previous = self.store.get_secret(id)?;
        self.store.remove_secret(id)?;
        let mut updated = self.saved.clone();
        updated.networks.retain(|n| n.id != id);
        if let Err(e) = self.store.save(&updated) {
            let _ = self.store.put_secret(id, &previous);
            return Err(e);
        }
        self.saved = updated;
        if self.lifecycle.snapshot().network_id.as_deref() == Some(id) {
            self.disconnect()?;
        }
        Ok(())
    }
    pub fn replace_credentials(&mut self, id: &str) -> Result<Network> {
        if self.lifecycle.is_active(id) {
            return Err(Error::Busy);
        }
        let original = self.network(id)?.clone();
        let label = format!(
            "{} (replacement)",
            original.label.chars().take(48).collect::<String>()
        );
        let network = Network {
            id: random_hex(16)?,
            label,
            ..original
        };
        // A replacement intentionally shares the addressing proposal. One active
        // network and explicit migration are required; the old network is retained.
        self.insert(network.clone(), Secret::generate()?)?;
        Ok(network)
    }
    pub fn save_preferences(&mut self, preferences: Preferences) -> Result<()> {
        validate_label(&preferences.nickname)?;
        let mut updated = self.saved.clone();
        updated.preferences = preferences;
        self.store.save(&updated)?;
        self.saved = updated;
        Ok(())
    }
    pub fn connect(&mut self, id: &str) -> Result<Snapshot> {
        let network = self.network(id)?.clone();
        let generation = self.lifecycle.begin_start(id)?;
        self.active_generation = Some(generation);
        self.events.record(&self.lifecycle.snapshot());
        let result = if self.runtime.as_ref().is_some_and(|r| r.available()) {
            self.store.get_secret(id).and_then(|credential| {
                let request = helper::HelperRequest::Start {
                    protocol: 1,
                    session_id: random_hex(16)?,
                    network,
                    credential,
                    nickname: self.saved.preferences.nickname.clone(),
                };
                self.runtime
                    .as_mut()
                    .ok_or(Error::HelperUnavailable)?
                    .start(request)
            })
        } else {
            Err(Error::HelperUnavailable)
        };
        if let Err(failure) = result {
            self.lifecycle.fail(generation, failure)?;
        }
        let state = self.lifecycle.snapshot();
        self.events.record(&state);
        Ok(state)
    }
    pub fn disconnect(&mut self) -> Result<Snapshot> {
        let generation = self.lifecycle.begin_stop()?;
        self.active_generation = Some(generation);
        self.events.record(&self.lifecycle.snapshot());
        let pending = match self.runtime.as_mut() {
            Some(runtime) => runtime.stop()?,
            None => false,
        };
        if !pending {
            self.lifecycle.finish_stop(generation)?;
            self.active_generation = None;
        }
        let state = self.lifecycle.snapshot();
        self.events.record(&state);
        Ok(state)
    }
    pub fn diagnostics(&self) -> DiagnosticReport {
        self.events.export(&self.lifecycle.snapshot())
    }
}
