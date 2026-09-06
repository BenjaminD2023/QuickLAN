//! Native, real-interface CI acceptance; never runs on a user's machine implicitly.
use quicklan_core::{
    helper::HelperRequest,
    model::{random_hex, Network, Policy, Secret},
    protocol::{self, EngineReply},
    system_routes,
};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

fn wait(child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(12);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "engine exit failed");
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    panic!("engine did not cleanly stop");
}
fn routes() -> Vec<String> {
    let mut routes = system_routes::read()
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    routes.sort();
    routes
}
fn start(
    path: &PathBuf,
) -> (
    Child,
    quicklan_ipc::LocalListener,
    quicklan_ipc::LocalStream,
    String,
    String,
) {
    let listener = quicklan_ipc::LocalListener::bind().unwrap();
    let mut child = Command::new(path)
        .args([
            "--ipc",
            &listener.endpoint(),
            "--parent-pid",
            &std::process::id().to_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stream = match listener.accept(child.id(), Duration::from_secs(10)) {
        Ok(stream) => stream,
        Err(_) => {
            let _ = child.kill();
            panic!("native helper IPC authentication failed");
        }
    };
    assert!(matches!(
        protocol::receive(&mut stream).unwrap(),
        EngineReply::Ready { protocol: 1, .. }
    ));
    let session = random_hex(16).unwrap();
    let subnet = "10.73.42.0/24".to_owned();
    let request = HelperRequest::Start {
        protocol: 1,
        session_id: session.clone(),
        network: Network {
            id: random_hex(16).unwrap(),
            label: "CI native network".into(),
            subnet: subnet.clone(),
            policy: Policy::Manual,
            bootstrap: vec![],
        },
        credential: Secret::generate().unwrap(),
        nickname: "CI native device".into(),
    };
    protocol::send(&mut stream, &request).unwrap();
    let reply: EngineReply = protocol::receive(&mut stream).unwrap();
    let EngineReply::State {
        virtual_ip: Some(ip),
        ..
    } = reply
    else {
        let _ = child.kill();
        panic!("engine failed to create a real virtual adapter: {reply:?}");
    };
    assert!(ip.starts_with("10.73.42."));
    assert!(
        system_routes::read()
            .unwrap()
            .iter()
            .any(|route| route.prefix_len() == 24 && route.network().to_string() == "10.73.42.0"),
        "virtual network OS route missing"
    );
    (child, listener, stream, session, ip)
}
fn main() {
    assert_eq!(
        std::env::var("CI").as_deref(),
        Ok("true"),
        "disposable native CI only"
    );
    assert!(quicklan_ipc::is_elevated(), "native CI privileges required");
    let path = PathBuf::from(std::env::args().nth(1).expect("engine path"))
        .canonicalize()
        .unwrap();
    let before = routes();
    let (mut child, listener, mut stream, session, _) = start(&path);
    protocol::send(
        &mut stream,
        &HelperRequest::Stop {
            protocol: 1,
            session_id: session,
        },
    )
    .unwrap();
    assert!(matches!(
        protocol::receive(&mut stream).unwrap(),
        EngineReply::Stopped {}
    ));
    quicklan_ipc::disconnect(&stream);
    drop(stream);
    drop(listener);
    wait(&mut child);
    assert_eq!(before, routes(), "routes changed after normal stop");
    let (mut child, listener, stream, _, _) = start(&path);
    quicklan_ipc::disconnect(&stream);
    drop(stream);
    drop(listener);
    wait(&mut child);
    assert_eq!(before, routes(), "routes changed after controller loss");
    println!("Native IPC, real virtual interface, stop, controller-loss cleanup and route restoration passed.");
}
