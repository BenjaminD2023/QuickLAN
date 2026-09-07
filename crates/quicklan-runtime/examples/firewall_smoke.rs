//! Live OS mutation acceptance: exclusively for a disposable hosted CI machine.
use quicklan_runtime::firewall;
fn main() {
    assert_eq!(
        std::env::var("CI").as_deref(),
        Ok("true"),
        "CI-only firewall test"
    );
    let before = firewall::status().expect("read firewall");
    assert_eq!(
        firewall::set_enabled(false, false).unwrap_err(),
        "firewall_confirmation_required"
    );
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(firewall::status().unwrap()).unwrap()
    );
    let initially_enabled = firewall::set_enabled(true, false).expect("establish enabled baseline");
    assert!(initially_enabled.profiles.iter().all(|p| p.enabled));
    let disabled = firewall::set_enabled(false, true)
        .expect("disable through actual runtime authorization path");
    assert!(disabled.profiles.iter().all(|p| !p.enabled));
    let enabled = firewall::set_enabled(true, false)
        .expect("enable through actual runtime authorization path");
    assert!(enabled.profiles.iter().all(|p| p.enabled));
    println!(
        "{}",
        serde_json::json!({"before":before,"initially_enabled":initially_enabled,"disabled":disabled,"enabled":enabled,"confirmation_gate":true,"runtime_path":true})
    );
}
