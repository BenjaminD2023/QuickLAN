//! Supervises a verified elevated engine while the Tauri application stays unprivileged.
mod elevation;
use quicklan_core::{
    adapter::verify_bytes,
    error::{Error, Result},
    helper::HelperRequest,
    protocol::{self, EngineReply},
    runtime::{NetworkRuntime, RuntimeEvent},
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
    time::Duration,
};

pub struct DesktopRuntime {
    engine: PathBuf,
    digest: String,
    available: bool,
    worker: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    tx: mpsc::Sender<RuntimeEvent>,
    rx: mpsc::Receiver<RuntimeEvent>,
}
impl DesktopRuntime {
    pub fn new(engine: PathBuf, digest: String) -> Self {
        let available = !digest.is_empty() && verified(&engine, &digest).is_ok();
        let (tx, rx) = mpsc::channel();
        Self {
            engine,
            digest,
            available,
            worker: None,
            stop: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        }
    }
}
fn verified(path: &std::path::Path, digest: &str) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| Error::HelperUnavailable)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 150_000_000 {
        return Err(Error::IncompatibleCore);
    }
    let bytes = std::fs::read(path).map_err(|_| Error::IncompatibleCore)?;
    verify_bytes(&bytes, digest)
}
impl NetworkRuntime for DesktopRuntime {
    fn available(&self) -> bool {
        self.available
    }
    fn start(&mut self, request: HelperRequest) -> Result<()> {
        if let Some(worker) = self.worker.take() {
            if !worker.is_finished() {
                self.worker = Some(worker);
                return Err(Error::Busy);
            }
            worker.join().map_err(|_| Error::CoreFailed)?;
        }
        while self.rx.try_recv().is_ok() {}
        verified(&self.engine, &self.digest)?;
        self.stop = Arc::new(AtomicBool::new(false));
        let stop = self.stop.clone();
        let tx = self.tx.clone();
        let engine = self.engine.clone();
        let digest = self.digest.clone();
        self.worker = Some(std::thread::spawn(move || {
            let result = run(engine, digest, request, &stop, &tx);
            if let Err(error) = result {
                let _ = tx.send(RuntimeEvent::Failed(error));
            }
        }));
        Ok(())
    }
    fn stop(&mut self) -> Result<bool> {
        self.stop.store(true, Ordering::SeqCst);
        Ok(self.worker.as_ref().is_some_and(|w| !w.is_finished()))
    }
    fn poll(&mut self) -> Vec<RuntimeEvent> {
        self.rx.try_iter().collect()
    }
}
impl Drop for DesktopRuntime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

fn run(
    engine: PathBuf,
    digest: String,
    request: HelperRequest,
    stop: &AtomicBool,
    events: &mpsc::Sender<RuntimeEvent>,
) -> Result<()> {
    let session = match &request {
        HelperRequest::Start { session_id, .. } => session_id.clone(),
        _ => return Err(Error::Unauthorized),
    };
    let listener = quicklan_ipc::LocalListener::bind().map_err(|_| Error::UnsafePath)?;
    let child = elevation::launch(&engine, &digest, &listener.endpoint(), std::process::id())?;
    let result = (|| {
        let mut stream = listener
            .accept(child.pid, Duration::from_secs(10))
            .map_err(|_| Error::Unauthorized)?;
        let ready: EngineReply = protocol::receive(&mut stream).map_err(|_| Error::CoreFailed)?;
        if !matches!(ready, EngineReply::Ready { protocol: 1, ref engine_version } if engine_version == "quicklan-engine-0.2.0-easytier-2.6.4")
        {
            return Err(Error::IncompatibleCore);
        }
        if stop.load(Ordering::SeqCst) {
            return Ok(());
        }
        let _ = events.send(RuntimeEvent::Joining);
        protocol::send(&mut stream, &request).map_err(|_| Error::CoreFailed)?;
        drop(request);
        let mut reader = stream.try_clone().map_err(|_| Error::CoreFailed)?;
        let (reply_tx, reply_rx) = mpsc::sync_channel(2);
        let reader_thread = std::thread::spawn(move || loop {
            let reply = protocol::receive::<EngineReply>(&mut reader);
            let failed = reply.is_err();
            if reply_tx.send(reply).is_err() || failed {
                break;
            }
        });
        let mut stop_sent = false;
        let mut next_status = None;
        let mut deadline = std::time::Instant::now() + Duration::from_secs(60);
        let outcome = loop {
            if stop.load(Ordering::SeqCst) && !stop_sent {
                if protocol::send(
                    &mut stream,
                    &HelperRequest::Stop {
                        protocol: 1,
                        session_id: session.clone(),
                    },
                )
                .is_err()
                {
                    break Err(Error::CoreFailed);
                }
                stop_sent = true;
                next_status = None;
                deadline = std::time::Instant::now() + Duration::from_secs(10);
            }
            if !stop_sent && next_status.is_some_and(|when| std::time::Instant::now() >= when) {
                if protocol::send(&mut stream, &HelperRequest::Status { protocol: 1 }).is_err() {
                    break Err(Error::CoreFailed);
                }
                next_status = None;
                deadline = std::time::Instant::now() + Duration::from_secs(20);
            }
            match reply_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(EngineReply::State { virtual_ip, peers })) => {
                    if !stop_sent {
                        let _ = events.send(RuntimeEvent::State { virtual_ip, peers });
                        next_status = Some(std::time::Instant::now() + Duration::from_secs(2));
                    }
                }
                Ok(Ok(EngineReply::Stopped {})) => break Ok(()),
                Ok(Ok(EngineReply::Failed { code })) => break Err(code),
                Ok(Ok(EngineReply::Ready { .. })) => break Err(Error::UnsupportedCoreOutput),
                Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break Err(Error::CoreFailed)
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
            }
            if std::time::Instant::now() >= deadline {
                break Err(Error::CoreFailed);
            }
        };
        // Cancel the cloned reader too, otherwise it would keep IPC open after
        // an error and prevent the engine from observing the desktop's EOF.
        quicklan_ipc::disconnect(&stream);
        drop(reply_rx);
        let _ = reader_thread.join();
        outcome
    })();
    // Dropping the stream also handles every error path. The engine treats EOF
    // as mandatory teardown. Never acknowledge a disconnect while it is alive.
    if !child.wait(Duration::from_secs(20)) {
        let _ = events.send(RuntimeEvent::Failed(Error::CoreFailed));
        // Keep ownership and refuse another engine until the previous elevated
        // process is confirmed dead. A failed UI state is not proof of cleanup.
        while !child.wait(Duration::from_secs(1)) {}
        return Err(Error::CoreFailed);
    }
    result?;
    let _ = events.send(RuntimeEvent::Stopped);
    Ok(())
}
