use crate::network::NetworkSession;
use crate::platform::AndroidPlatform;
use quicklan_core::{
    error::{Error, Result},
    helper::HelperRequest,
    protocol::EngineReply,
    runtime::{NetworkRuntime, RuntimeEvent},
};
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};

pub struct AndroidRuntime {
    platform: AndroidPlatform,
    tokio: tokio::runtime::Handle,
    worker: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<RuntimeEvent>>>,
}

// Keep at most the joining transition and latest observation while the UI is
// backgrounded. Peer snapshots must never form an unbounded background queue.
fn publish(events: &Mutex<Vec<RuntimeEvent>>, event: RuntimeEvent) {
    let mut queued = events.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if matches!(event, RuntimeEvent::State { .. }) {
        queued.retain(|old| matches!(old, RuntimeEvent::Joining));
    } else {
        queued.clear();
    }
    queued.push(event);
}

impl AndroidRuntime {
    pub fn new(platform: AndroidPlatform, tokio: tokio::runtime::Handle) -> Self {
        Self {
            platform,
            tokio,
            worker: None,
            stop: Arc::new(AtomicBool::new(false)),
            events: Arc::new(Mutex::new(Vec::with_capacity(2))),
        }
    }

    fn join(&mut self) -> Result<()> {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| Error::CoreFailed)?;
        }
        Ok(())
    }
}

impl NetworkRuntime for AndroidRuntime {
    fn available(&self) -> bool { true }

    fn start(&mut self, request: HelperRequest) -> Result<()> {
        if self.worker.as_ref().is_some_and(|worker| !worker.is_finished()) {
            return Err(Error::Busy);
        }
        self.join()?;
        self.events.lock().map_err(|_| Error::CoreFailed)?.clear();
        self.stop = Arc::new(AtomicBool::new(false));
        let stop = self.stop.clone();
        let events = self.events.clone();
        let platform = self.platform.clone();
        let tokio = self.tokio.clone();
        self.worker = Some(std::thread::spawn(move || {
            if let Err(error) = run(tokio, platform, request, &stop, &events) {
                publish(&events, RuntimeEvent::Failed(error));
            }
        }));
        Ok(())
    }

    fn stop(&mut self) -> Result<bool> {
        // A successful disconnect is an actual worker/TUN shutdown acknowledgement.
        self.join()?;
        self.events.lock().map_err(|_| Error::CoreFailed)?.clear();
        Ok(false)
    }

    fn poll(&mut self) -> Vec<RuntimeEvent> {
        std::mem::take(&mut *self.events.lock().unwrap_or_else(|p| p.into_inner()))
    }
}

impl Drop for AndroidRuntime {
    fn drop(&mut self) { let _ = self.join(); }
}

fn run(
    tokio: tokio::runtime::Handle,
    platform: AndroidPlatform,
    request: HelperRequest,
    stop: &AtomicBool,
    events: &Mutex<Vec<RuntimeEvent>>,
) -> Result<()> {
    // EasyTier's Instance::drop spawns cleanup tasks, including after block_on.
    let _entered = tokio.enter();
    let HelperRequest::Start { session_id, network, credential, nickname, .. } = request else {
        return Err(Error::Unauthorized);
    };
    if stop.load(Ordering::SeqCst) { return Ok(()); }
    publish(events, RuntimeEvent::Joining);
    let started = tokio.block_on(NetworkSession::start_mobile(
        &network, &credential, &nickname, &session_id, Arc::new(platform), stop,
    ));
    drop(credential);
    let mut session = match started {
        Ok(session) => session,
        Err(_) if stop.load(Ordering::SeqCst) => return Ok(()),
        Err(error) => return Err(error),
    };
    let outcome = tokio.block_on(async {
        loop {
            if stop.load(Ordering::SeqCst) { break Ok(()); }
            match tokio::time::timeout(Duration::from_secs(5), session.state()).await {
                Ok(Ok(EngineReply::State { virtual_ip, peers })) => {
                    if !stop.load(Ordering::SeqCst) {
                        publish(events, RuntimeEvent::State { virtual_ip, peers });
                    }
                }
                Ok(Err(error)) => break Err(error),
                _ => break Err(Error::CoreFailed),
            }
            for _ in 0..20 {
                if stop.load(Ordering::SeqCst) { break; }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    });
    tokio.block_on(session.stop());
    drop(session);
    if stop.load(Ordering::SeqCst) { Ok(()) } else { outcome }
}
