use crate::platform::AndroidPlatform;
use crate::store::AndroidStore;
use quicklan_core::{
    app::App,
    error::{Error, Result},
    model::{Bootstrap, Policy, Preferences},
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateArgs {
    label: String,
    nickname: String,
    subnet: String,
    policy: Policy,
    bootstrap: Vec<Bootstrap>,
    assistance_accepted: bool,
}

#[derive(Deserialize)]
struct TokenArg {
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcceptArgs {
    ticket: String,
    trusted: bool,
    assistance_accepted: bool,
}

#[derive(Deserialize)]
struct RenameArgs {
    id: String,
    label: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsArgs {
    id: String,
    policy: Policy,
    bootstrap: Vec<Bootstrap>,
    assistance_accepted: bool,
}

#[derive(Deserialize)]
struct PreferencesArgs {
    preferences: Preferences,
}

#[derive(Deserialize)]
struct IpArg {
    ip: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProbeArgs {
    peer_id: String,
    port: u16,
}


pub fn invoke(
    app: &mut App<AndroidStore>,
    platform: &AndroidPlatform,
    command: &str,
    args: &str,
) -> std::result::Result<Value, String> {
    if command == "get_firewall_status" {
        return Ok(firewall_status());
    }
    if command == "set_firewall_enabled" {
        return Err("firewall_unsupported".into());
    }
    dispatch(app, platform, command, args).map_err(error_code)
}

fn dispatch(
    app: &mut App<AndroidStore>,
    platform: &AndroidPlatform,
    command: &str,
    args: &str,
) -> Result<Value> {
    let args = parse_value(args)?;
    match command {
        "get_state" => {
            app.refresh()?;
            to_value(app.view())
        }
        "create_network" => {
            let args: CreateArgs = parse(args)?;
            let routes = quicklan_core::system_routes::read()?;
            let saved = app
                .view()
                .saved
                .networks
                .into_iter()
                .map(|n| n.subnet)
                .collect::<Vec<_>>();
            let subnet = if args.subnet.is_empty() {
                quicklan_core::routes::choose_subnet(&routes, &saved)?
            } else {
                quicklan_core::routes::check_conflicts(&args.subnet, &routes, &saved)?;
                args.subnet
            };
            to_value(app.create_with_nickname(
                args.label,
                subnet,
                args.policy,
                args.bootstrap,
                args.assistance_accepted,
                args.nickname,
            )?)
        }
        "preview_invitation" => {
            let args: TokenArg = parse(args)?;
            let token = zeroize::Zeroizing::new(args.token);
            to_value(app.preview(&token)?)
        }
        "accept_invitation" => {
            let args: AcceptArgs = parse(args)?;
            to_value(app.accept(&args.ticket, args.trusted, args.assistance_accepted)?)
        }
        "cancel_invitation" => {
            app.cancel_preview();
            Ok(Value::Null)
        }
        "rename_network" => {
            let args: RenameArgs = parse(args)?;
            app.rename(&args.id, args.label)?;
            Ok(Value::Null)
        }
        "forget_network" => {
            let args: IdArg = parse(args)?;
            app.forget(&args.id)?;
            Ok(Value::Null)
        }
        "replace_credentials" => {
            let args: IdArg = parse(args)?;
            to_value(app.replace_credentials(&args.id)?)
        }
        "connect_network" => {
            let args: IdArg = parse(args)?;
            to_value(app.connect(&args.id)?)
        }
        "disconnect_network" => {
            app.disconnect()?;
            to_value(app.view().connection)
        }
        "save_preferences" => {
            let args: PreferencesArgs = parse(args)?;
            app.save_preferences(args.preferences)?;
            Ok(Value::Null)
        }
        "update_settings" => {
            let args: SettingsArgs = parse(args)?;
            app.update_settings(
                &args.id,
                args.policy,
                args.bootstrap,
                args.assistance_accepted,
            )?;
            Ok(Value::Null)
        }
        "get_diagnostics" => to_value(app.diagnostics()),
        "copy_invitation" => {
            let args: IdArg = parse(args)?;
            let token = app.invitation(&args.id)?;
            platform.copy_text(token.as_str(), true)?;
            Ok(Value::Null)
        }
        "copy_diagnostics" => {
            let report = app.diagnostics();
            let text = serde_json::to_string_pretty(&report).map_err(|_| Error::CoreFailed)?;
            platform.copy_text(&text, false)?;
            Ok(Value::Null)
        }
        "copy_virtual_ip" => {
            let args: IpArg = parse(args)?;
            let snapshot = app.view().connection;
            if snapshot.virtual_ip.as_deref() != Some(&args.ip)
                && !snapshot
                    .peers
                    .iter()
                    .any(|peer| peer.virtual_ip.as_deref() == Some(&args.ip))
            {
                return Err(Error::InvalidService);
            }
            platform.copy_text(&args.ip, true)?;
            Ok(Value::Null)
        }
        "probe_service" => {
            let args: ProbeArgs = parse(args)?;
            // JNI commands share the application mutex, so only one probe runs.
            let endpoint = app.service_endpoint(&args.peer_id, args.port)?;
            to_value(quicklan_core::service::probe(endpoint))
        }
        "get_local_endpoints" => platform.local_endpoints(),
        "quit_app" => {
            app.disconnect()?;
            Ok(Value::Null)
        }
        _ => Err(Error::Unauthorized),
    }
}


fn parse_value(args: &str) -> Result<Value> {
    if args.len() > 65_536 {
        return Err(Error::Unauthorized);
    }
    if args.is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(args).map_err(|_| Error::Unauthorized)
}

fn parse<T: for<'de> Deserialize<'de>>(args: Value) -> Result<T> {
    serde_json::from_value(args).map_err(|_| Error::Unauthorized)
}

fn to_value<T: serde::Serialize>(value: T) -> Result<Value> {
    serde_json::to_value(value).map_err(|_| Error::CoreFailed)
}

fn firewall_status() -> Value {
    json!({
        "platform": "android",
        "enabled": null,
        "can_toggle": false,
        "profiles": []
    })
}

fn error_code(error: Error) -> String {
    match serde_json::to_value(error) {
        Ok(Value::String(code)) => code,
        _ => "core_failed".into(),
    }
}
