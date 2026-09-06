//! Separate GPL-3.0 networking executable. No web UI and no TCP management listener.
mod network;
use quicklan_core::{
    error::Error,
    helper::{HelperRequest, MAX_REQUEST_BYTES},
    protocol::{self, EngineReply},
};
use std::{
    io::{Read, Write},
    time::Duration,
};

const ENGINE_VERSION: &str = "quicklan-engine-0.2.0-easytier-2.6.4";

async fn serve(reader: impl Read + Send + 'static, mut writer: impl Write) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || {
        let mut reader = reader;
        loop {
            let request = protocol::read_frame(&mut reader, MAX_REQUEST_BYTES)
                .ok()
                .and_then(|bytes| HelperRequest::parse(&bytes).ok());
            let Some(request) = request else {
                break;
            };
            if tx.blocking_send(request).is_err() {
                break;
            }
        }
    });
    if protocol::send(
        &mut writer,
        &EngineReply::Ready {
            protocol: protocol::VERSION,
            engine_version: ENGINE_VERSION.into(),
        },
    )
    .is_err()
    {
        return;
    }
    let Some(HelperRequest::Start {
        session_id,
        network,
        credential,
        nickname,
        ..
    }) = tokio::time::timeout(Duration::from_secs(15), rx.recv())
        .await
        .ok()
        .flatten()
    else {
        return;
    };
    let start = tokio::select! {
        started = network::NetworkSession::start(&network, &credential, &nickname, &session_id) => started,
        _ = rx.recv() => {
            // Cancellation/EOF while the driver is starting must not leave a
            // networking task behind after the controlling desktop has closed.
            let _ = protocol::send(&mut writer, &EngineReply::Stopped {});
            return;
        }
    };
    let mut session = match start {
        Ok(session) => session,
        Err(code) => {
            let _ = protocol::send(&mut writer, &EngineReply::Failed { code });
            return;
        }
    };
    drop(credential);
    loop {
        match session.state().await {
            Ok(reply) => {
                if protocol::send(&mut writer, &reply).is_err() {
                    break;
                }
            }
            Err(code) => {
                let _ = protocol::send(&mut writer, &EngineReply::Failed { code });
                break;
            }
        }
        // The desktop sends status heartbeats. Losing the controlling process,
        // malformed input, stalled IPC or missing heartbeats always tears down.
        match tokio::time::timeout(Duration::from_secs(15), rx.recv()).await {
            Ok(Some(HelperRequest::Status { .. })) => (),
            Ok(Some(HelperRequest::Stop {
                session_id: stop_id,
                ..
            })) if stop_id == session_id => break,
            _ => {
                let _ = protocol::send(
                    &mut writer,
                    &EngineReply::Failed {
                        code: Error::Unauthorized,
                    },
                );
                break;
            }
        }
    }
    session.stop().await;
    drop(session);
    let _ = protocol::send(&mut writer, &EngineReply::Stopped {});
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("{ENGINE_VERSION}");
        return;
    }
    // Raw core tracing is intentionally never installed: it can contain secrets.
    // The stdio transport exists only in explicitly compiled integration-test builds.
    #[cfg(feature = "lab")]
    if std::env::args().nth(1).as_deref() == Some("--stdio-lab") {
        serve(std::io::stdin(), std::io::stdout()).await;
        return;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 5
        && args[1] == "--ipc"
        && args[3] == "--parent-pid"
        && quicklan_ipc::is_elevated()
    {
        let Ok(parent) = args[4].parse::<u32>() else {
            std::process::exit(2);
        };
        if parent <= 1 || parent == std::process::id() {
            std::process::exit(2);
        }
        if let Ok(stream) = quicklan_ipc::connect(&args[2], parent) {
            if let Ok(reader) = stream.try_clone() {
                serve(reader, stream).await;
                return;
            }
        }
    }
    std::process::exit(2);
}
