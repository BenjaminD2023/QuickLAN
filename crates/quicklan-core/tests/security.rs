use proptest::prelude::*;
use quicklan_core::{
    app::App,
    error::Error,
    helper::HelperRequest,
    invitation::{validate_endpoint, Invitation},
    lifecycle::{Lifecycle, Phase},
    model::*,
    routes::*,
    storage::{decode_state, encode_state, Store},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Memory {
    state: SavedState,
    secrets: HashMap<String, Secret>,
    fail_save: bool,
}
#[derive(Clone, Default)]
struct TestStore(Arc<Mutex<Memory>>);
impl Store for TestStore {
    fn load(&self) -> quicklan_core::error::Result<SavedState> {
        Ok(self.0.lock().unwrap().state.clone())
    }
    fn save(&mut self, s: &SavedState) -> quicklan_core::error::Result<()> {
        let mut m = self.0.lock().unwrap();
        if m.fail_save {
            return Err(Error::StorageUnavailable);
        }
        m.state = s.clone();
        Ok(())
    }
    fn get_secret(&self, id: &str) -> quicklan_core::error::Result<Secret> {
        self.0
            .lock()
            .unwrap()
            .secrets
            .get(id)
            .cloned()
            .ok_or(Error::StorageUnavailable)
    }
    fn put_secret(&mut self, id: &str, s: &Secret) -> quicklan_core::error::Result<()> {
        self.0.lock().unwrap().secrets.insert(id.into(), s.clone());
        Ok(())
    }
    fn remove_secret(&mut self, id: &str) -> quicklan_core::error::Result<()> {
        self.0.lock().unwrap().secrets.remove(id);
        Ok(())
    }
}
fn network() -> Network {
    Network {
        id: random_hex(16).unwrap(),
        label: "Friday night".into(),
        subnet: "10.73.42.0/24".into(),
        policy: Policy::Manual,
        bootstrap: vec![],
    }
}
fn invitation() -> Invitation {
    Invitation::new(network(), Secret::generate().unwrap())
}

#[test]
fn bearer_round_trip_and_redacted_debug() {
    let invite = invitation();
    let token = invite.encode().unwrap();
    let decoded = Invitation::parse(&token).unwrap();
    assert_eq!(decoded.network, invite.network);
    assert_eq!(decoded.credential.expose(), invite.credential.expose());
    assert!(!format!("{invite:?}").contains(invite.credential.expose()));
    assert_eq!(invite.credential.expose().len(), 64);
    assert_eq!(invite.network.id.len(), 32);
}
#[test]
fn reject_version_extension_duplicate_fields_and_large_tokens() {
    let mut i = invitation();
    i.protocol = "future".into();
    assert_eq!(i.encode().unwrap_err(), Error::UnsupportedVersion);
    assert!(Invitation::parse(&"x".repeat(9000)).is_err());
    use base64::Engine;
    let i = invitation();
    let mut val = serde_json::to_value(i).unwrap();
    val["command"] = serde_json::json!("sh");
    let t = format!(
        "quicklan1:{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&val).unwrap())
    );
    assert!(Invitation::parse(&t).is_err());
    assert!(HelperRequest::parse(br#"{"operation":"status","protocol":1,"protocol":1}"#).is_err());
}
#[test]
fn hostile_endpoints() {
    for e in [
        "file:///etc/passwd",
        "https://example.com/config",
        "tcp://user:pass@host:10",
        "tcp://127.0.0.1:22",
        "udp://0.0.0.0:80",
        "tcp://192.168.1.1:0",
        "tcp://192.168.1.1:22/path",
        "tcp://192.168.1.1:22?exec=a",
        "tcp://192.168.1.1:22#x",
        "tcp://$(whoami):22",
        "tcp://192.168.1.1:22\n--exit-node",
        "tcp://[::ffff:127.0.0.1]:22",
        "udp://[fe80::1%en0]:80",
        "tcp://192.168.1.1",
        "tcp://2130706433:22",
        "tcp://192.168.1.1:22/",
    ] {
        assert!(validate_endpoint(e, Policy::Manual).is_err(), "{e}");
    }
}
#[test]
fn accepted_endpoints_are_only_explicit_connections() {
    assert!(validate_endpoint("tcp://192.168.1.12:11010", Policy::Manual).is_ok());
    assert!(validate_endpoint("udp://[fd12::2]:11010", Policy::Manual).is_ok());
    assert!(validate_endpoint("tcp://relay.example.com:11010", Policy::Assisted).is_ok());
    assert!(validate_endpoint("tcp://relay.example.com:11010", Policy::Manual).is_err());
}
#[test]
fn safe_ranges_and_coordinated_conflict() {
    for s in [
        "0.0.0.0/0",
        "100.64.0.0/24",
        "8.8.8.0/24",
        "10.73.42.1/24",
        "10.73.42.0/16",
        "127.0.0.0/24",
    ] {
        assert!(validate_subnet(s).is_err());
    }
    assert!(check_conflicts("10.73.42.0/24", &["0.0.0.0/0".parse().unwrap()], &[]).is_ok());
    assert_eq!(
        check_conflicts("10.73.42.0/24", &["10.0.0.0/8".parse().unwrap()], &[]),
        Err(Error::RouteConflict)
    );
    assert!(validate_advertisement(
        "10.73.42.0/24",
        "10.73.42.2".parse().unwrap(),
        &["0.0.0.0/0".into()]
    )
    .is_err());
}
#[test]
fn confirmation_is_bound_to_preview_and_no_implicit_connection() {
    let mut app = App::new(TestStore::default()).unwrap();
    let t = invitation().encode().unwrap();
    let p = app.preview(&t).unwrap();
    assert_eq!(
        app.accept(&p.ticket, false, false).unwrap_err(),
        Error::ConfirmationRequired
    );
    assert!(app.accept(&random_hex(16).unwrap(), true, false).is_err());
    assert!(app.accept(&p.ticket, true, false).is_ok());
    assert_eq!(app.view().connection.phase, Phase::Disconnected);
    assert!(app.accept(&p.ticket, true, false).is_err());
}
#[test]
fn failed_storage_rolls_back_create() {
    let store = TestStore::default();
    let mut app = App::new(store.clone()).unwrap();
    store.0.lock().unwrap().fail_save = true;
    assert!(app
        .create(
            "Friends".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false
        )
        .is_err());
    assert!(app.view().saved.networks.is_empty());
    assert!(store.0.lock().unwrap().secrets.is_empty());
}
#[test]
fn credentials_not_in_saved_state_or_diagnostics() {
    let store = TestStore::default();
    let mut app = App::new(store.clone()).unwrap();
    let n = app
        .create(
            "Friends".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false,
        )
        .unwrap();
    let secret = store.get_secret(&n.id).unwrap();
    let bytes = encode_state(&app.view().saved).unwrap();
    assert!(!String::from_utf8(bytes.clone())
        .unwrap()
        .contains(secret.expose()));
    assert_eq!(decode_state(&bytes).unwrap().networks.len(), 1);
    app.connect(&n.id).unwrap();
    let d = serde_json::to_string(&app.diagnostics()).unwrap();
    for forbidden in [secret.expose(), &n.id, &n.label, &n.subnet] {
        assert!(!d.contains(forbidden));
    }
}
#[test]
fn stock_core_cannot_start_elevated_and_errors_remain_truthful() {
    let mut app = App::new(TestStore::default()).unwrap();
    let n = app
        .create(
            "Friends".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false,
        )
        .unwrap();
    let s = app.connect(&n.id).unwrap();
    assert_eq!(s.phase, Phase::Failed);
    assert_eq!(s.error, Some(Error::HelperUnavailable));
    assert!(s.peers.is_empty());
    assert!(s.virtual_ip.is_none());
    assert_eq!(app.disconnect().unwrap().phase, Phase::Disconnected);
}
#[test]
fn replaced_credentials_leave_old_network_intact() {
    let mut app = App::new(TestStore::default()).unwrap();
    let n = app
        .create(
            "Friends".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false,
        )
        .unwrap();
    let old = app.invitation(&n.id).unwrap();
    let new = app.replace_credentials(&n.id).unwrap();
    assert_ne!(n.id, new.id);
    assert_eq!(app.invitation(&n.id).unwrap().as_str(), old.as_str());
    assert_eq!(app.view().saved.networks.len(), 2);
    app.forget(&new.id).unwrap();
    assert!(app.network(&n.id).is_ok());
}
#[test]
fn direct_only_cannot_be_enabled_through_request() {
    let mut n = network();
    n.policy = Policy::DirectOnly;
    assert_eq!(n.validate(), Err(Error::UnsupportedPolicy));
}
#[test]
fn stale_results_cannot_resurrect_a_disconnected_engine() {
    let mut life = Lifecycle::default();
    let id = random_hex(16).unwrap();
    let old = life.begin_start(&id).unwrap();
    assert_eq!(life.begin_start(&id).unwrap_err(), Error::Busy);
    life.transition(old, Phase::Joining).unwrap();
    let stop = life.begin_stop().unwrap();
    life.finish_stop(stop).unwrap();
    let new = life.begin_start(&id).unwrap();
    assert!(life.transition(old, Phase::Connected).is_err());
    assert!(life.fail(old, Error::CoreFailed).is_err());
    assert!(life
        .observe(old, Some("10.73.42.1".into()), vec![])
        .is_err());
    assert!(new > old);
}
#[test]
fn reject_shell_paths_and_unknown_helper_commands() {
    for s in [
        r#"{"operation":"exec","program":"/bin/sh","protocol":1}"#,
        r#"{"operation":"status","protocol":1,"config_file":"/tmp/x"}"#,
        r#"{"operation":"status","protocol":2}"#,
    ] {
        assert!(HelperRequest::parse(s.as_bytes()).is_err());
    }
}
#[test]
fn parser_uses_captured_real_fixture_and_never_invents_zero_metrics() {
    let bytes = include_bytes!("../../../tests/fixtures/peer-0.json");
    let peers =
        quicklan_core::adapter::parse_peers(bytes, quicklan_core::CORE_VERSION, "10.73.42.0/24")
            .unwrap();
    assert_eq!(peers.len(), 2);
    assert_eq!(peers[0].latency_ms, None);
    assert!(peers.iter().all(|p| !p.identity_verified));
    let mut fixture: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    fixture[1]["lat_ms"] = serde_json::json!("0.00");
    let peers = quicklan_core::adapter::parse_peers(
        &serde_json::to_vec(&fixture).unwrap(),
        quicklan_core::CORE_VERSION,
        "10.73.42.0/24",
    )
    .unwrap();
    assert_eq!(peers[1].latency_ms, None);
    fixture[1]["new_schema"] = serde_json::json!(true);
    assert!(quicklan_core::adapter::parse_peers(
        &serde_json::to_vec(&fixture).unwrap(),
        quicklan_core::CORE_VERSION,
        "10.73.42.0/24"
    )
    .is_err());
}
#[test]
fn future_storage_schema_not_overwritten() {
    assert!(decode_state(br#"{"schema":999,"state":{"networks":[],"preferences":{"nickname":"My computer","theme":"system","language":"en","onboarding_complete":false}}}"#).is_err());
}
#[test]
fn rendered_configuration_is_data_not_instructions() {
    let i = invitation();
    let text =
        quicklan_core::adapter::candidate_config(&i.network, &i.credential, "quoted \" name")
            .unwrap();
    let parsed: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(parsed["hostname"].as_str(), Some("quoted \" name"));
    assert_eq!(parsed["routes"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["flags"]["disable_relay_data"].as_bool(), Some(true));
    assert_eq!(parsed["flags"]["accept_dns"].as_bool(), Some(false));
}
#[test]
fn bounded_diagnostics_events() {
    let mut l = quicklan_core::diagnostics::EventLog::default();
    for _ in 0..1000 {
        l.record(&Default::default());
    }
    assert_eq!(l.export(&Default::default()).events.len(), 100);
}

proptest! {
 #![proptest_config(ProptestConfig::with_cases(512))]
 #[test] fn invitation_fuzz_never_panics(bytes in prop::collection::vec(any::<u8>(),0..9000)) {if let Ok(s)=std::str::from_utf8(&bytes){let _=Invitation::parse(s);}}
 #[test] fn helper_fuzz_never_panics(bytes in prop::collection::vec(any::<u8>(),0..17000)){let _=HelperRequest::parse(&bytes);}
 #[test] fn endpoint_fuzz_never_panics(s in ".{0,300}"){let _=validate_endpoint(&s,Policy::Manual);}
}

#[cfg(feature = "os-vault")]
#[test]
fn atomic_metadata_permissions_and_symlink_rejection() {
    use quicklan_core::storage::OsStore;
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("private");
    let mut store = OsStore::new(dir.clone()).unwrap();
    store.save(&SavedState::default()).unwrap();
    assert_eq!(store.load().unwrap().networks.len(), 0);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{symlink, PermissionsExt};
        assert_eq!(
            std::fs::metadata(dir.join("networks-v1.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        std::fs::remove_file(dir.join("networks-v1.json")).unwrap();
        let unrelated = root.path().join("unrelated");
        std::fs::write(&unrelated, "unchanged").unwrap();
        symlink(&unrelated, dir.join("networks-v1.json")).unwrap();
        assert!(store.save(&SavedState::default()).is_err());
        assert_eq!(std::fs::read_to_string(unrelated).unwrap(), "unchanged");
    }
}

#[cfg(feature = "os-vault")]
#[test]
#[ignore = "Uses the real OS credential store; run explicitly on a native test machine"]
fn native_credential_round_trip() {
    use quicklan_core::storage::OsStore;
    let directory = tempfile::tempdir().unwrap();
    let mut store = OsStore::new(directory.path().join("quicklan-vault-test")).unwrap();
    let id = random_hex(16).unwrap();
    let secret = Secret::generate().unwrap();
    store.put_secret(&id, &secret).unwrap();
    let recovered = store.get_secret(&id);
    let removed = store.remove_secret(&id);
    assert_eq!(recovered.unwrap().expose(), secret.expose());
    removed.unwrap();
    assert!(store.get_secret(&id).is_err());
}

#[test]
fn create_nickname_update_rolls_back_with_failed_network_save() {
    let store = TestStore::default();
    let mut app = App::new(store.clone()).unwrap();
    store.0.lock().unwrap().fail_save = true;
    assert!(app
        .create_with_nickname(
            "Friends".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false,
            "Changed".into()
        )
        .is_err());
    assert_eq!(app.view().saved.preferences.nickname, "My computer");
}

#[test]
fn changed_assistance_requires_new_explicit_consent() {
    let mut app = App::new(TestStore::default()).unwrap();
    let n = app
        .create(
            "Friends".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false,
        )
        .unwrap();
    let node = Bootstrap {
        endpoint: "tcp://relay.example.com:11010".into(),
        operator: "Test operator".into(),
    };
    assert_eq!(
        app.update_settings(&n.id, Policy::Assisted, vec![node.clone()], false),
        Err(Error::AssistanceConsentRequired)
    );
    app.update_settings(&n.id, Policy::Assisted, vec![node], true)
        .unwrap();
    let invite = app.invitation(&n.id).unwrap();
    assert_eq!(
        Invitation::parse(&invite).unwrap().network.bootstrap.len(),
        1
    );
}

#[derive(Clone, Default)]
struct RuntimeFixture(Arc<Mutex<Vec<quicklan_core::runtime::RuntimeEvent>>>);
impl quicklan_core::runtime::NetworkRuntime for RuntimeFixture {
    fn available(&self) -> bool {
        true
    }
    fn start(&mut self, request: HelperRequest) -> quicklan_core::error::Result<()> {
        assert!(matches!(request, HelperRequest::Start { .. }));
        Ok(())
    }
    fn stop(&mut self) -> quicklan_core::error::Result<bool> {
        Ok(true)
    }
    fn poll(&mut self) -> Vec<quicklan_core::runtime::RuntimeEvent> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}
#[test]
fn live_runtime_requires_shutdown_ack_and_rejects_stale_peer_observations() {
    use quicklan_core::{
        adapter::{Peer, PeerPath},
        runtime::RuntimeEvent,
    };
    let runtime = RuntimeFixture::default();
    let mut app = App::new(TestStore::default())
        .unwrap()
        .with_runtime(Box::new(runtime.clone()));
    let network = app
        .create(
            "Test network".into(),
            "10.73.42.0/24".into(),
            Policy::Manual,
            vec![],
            false,
        )
        .unwrap();
    assert_eq!(app.connect(&network.id).unwrap().phase, Phase::Starting);
    assert!(app.connect(&network.id).is_err());
    runtime.0.lock().unwrap().extend([
        RuntimeEvent::Joining,
        RuntimeEvent::State {
            virtual_ip: Some("10.73.42.1".into()),
            peers: vec![Peer {
                id: "42".into(),
                nickname: "Friend".into(),
                virtual_ip: Some("10.73.42.2".into()),
                path: PeerPath::Direct,
                latency_ms: None,
                identity_verified: false,
            }],
        },
    ]);
    app.refresh().unwrap();
    assert_eq!(app.view().connection.phase, Phase::Connected);
    assert_eq!(
        app.service_endpoint("42", 25565).unwrap().to_string(),
        "10.73.42.2:25565"
    );
    assert!(app.service_endpoint("arbitrary-host", 80).is_err());
    assert!(app.service_endpoint("42", 0).is_err());
    assert_eq!(app.disconnect().unwrap().phase, Phase::Stopping);
    assert!(app.service_endpoint("42", 25565).is_err());
    runtime.0.lock().unwrap().push(RuntimeEvent::State {
        virtual_ip: Some("10.73.42.1".into()),
        peers: vec![],
    });
    app.refresh().unwrap();
    assert_eq!(app.view().connection.phase, Phase::Stopping);
    runtime.0.lock().unwrap().push(RuntimeEvent::Stopped);
    app.refresh().unwrap();
    assert_eq!(app.view().connection.phase, Phase::Disconnected);
    assert!(app.view().connection.virtual_ip.is_none());
}
