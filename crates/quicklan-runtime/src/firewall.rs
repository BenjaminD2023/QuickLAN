//! Explicit, administrator-authorized OS firewall controls. Never called by connect/quit.
use serde::Serialize;
use std::sync::Mutex;

static MUTATION: Mutex<()> = Mutex::new(());
#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub name: String,
    pub enabled: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub platform: &'static str,
    pub profiles: Vec<Profile>,
}
pub type Result<T> = std::result::Result<T, &'static str>;

pub fn status() -> Result<Status> {
    platform::status()
}
pub fn set_enabled(enabled: bool, confirmed: bool) -> Result<Status> {
    if !enabled && !confirmed {
        return Err("firewall_confirmation_required");
    }
    let _guard = MUTATION.try_lock().map_err(|_| "firewall_busy")?;
    // An unreadable state must never be interpreted as disabled.
    status()?;
    platform::set(enabled)?;
    for _ in 0..10 {
        let current = status()?;
        if !current.profiles.is_empty() && current.profiles.iter().all(|p| p.enabled == enabled) {
            return Ok(current);
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    Err("firewall_policy_blocked")
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::process::Command;
    const TOOL: &str = "/usr/libexec/ApplicationFirewall/socketfilterfw";
    pub fn status() -> Result<Status> {
        let out = Command::new(TOOL)
            .arg("--getglobalstate")
            .env_clear()
            .env("LC_ALL", "C")
            .output()
            .map_err(|_| "firewall_unavailable")?;
        if !out.status.success() {
            return Err("firewall_unavailable");
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let enabled = parse_state(&text)?;
        Ok(Status {
            platform: "macos",
            profiles: vec![Profile {
                name: "Application Firewall".into(),
                enabled,
            }],
        })
    }
    fn parse_state(text: &str) -> Result<bool> {
        if text.contains("(State = 0)") {
            Ok(false)
        } else if text.contains("(State = 1)") || text.contains("(State = 2)") {
            Ok(true)
        } else {
            Err("firewall_unavailable")
        }
    }
    pub fn set(enabled: bool) -> Result<()> {
        // Both commands are compile-time literals; renderer input never becomes shell text.
        let script = if enabled {
            "do shell script \"/usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LC_ALL=C /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on\" with administrator privileges"
        } else {
            "do shell script \"/usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin LC_ALL=C /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off\" with administrator privileges"
        };
        let out = Command::new("/usr/bin/osascript")
            .args(["-e", script])
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .output()
            .map_err(|_| "firewall_change_failed")?;
        if out.status.success() {
            Ok(())
        } else {
            Err("firewall_change_failed")
        }
    }
    #[cfg(test)]
    #[test]
    fn state_is_never_guessed() {
        assert_eq!(parse_state("Firewall is disabled. (State = 0)"), Ok(false));
        assert_eq!(parse_state("Firewall is enabled. (State = 1)"), Ok(true));
        assert_eq!(parse_state("Firewall is enabled. (State = 2)"), Ok(true));
        assert!(parse_state("Permission denied").is_err());
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::{mem::size_of, os::windows::process::CommandExt, path::PathBuf, process::Command};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, WAIT_OBJECT_0},
        System::{
            SystemInformation::GetSystemDirectoryW,
            Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE},
        },
        UI::{
            Shell::{
                ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
            },
            WindowsAndMessaging::SW_HIDE,
        },
    };
    fn system_dir() -> Result<PathBuf> {
        let mut buffer = [0u16; 32768];
        let len = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
        if len == 0 || len >= buffer.len() {
            return Err("firewall_unavailable");
        }
        Ok(PathBuf::from(
            String::from_utf16(&buffer[..len]).map_err(|_| "firewall_unavailable")?,
        ))
    }
    pub fn status() -> Result<Status> {
        let system = system_dir()?;
        let modules = system.join("WindowsPowerShell/v1.0/Modules");
        let module = modules.join("NetSecurity/NetSecurity.psd1");
        // System directory comes from the OS API, never an environment override.
        let quote = |p: &std::path::Path| p.to_string_lossy().replace('\'', "''");
        let script = format!("$ErrorActionPreference='Stop'; $env:PSModulePath='{}'; Import-Module '{}'; @(Get-NetFirewallProfile -PolicyStore ActiveStore | ForEach-Object {{ @{{name=[string]$_.Name; enabled=[int]$_.Enabled}} }}) | ConvertTo-Json -Compress", quote(&modules), quote(&module));
        let out = Command::new(system.join("WindowsPowerShell/v1.0/powershell.exe"))
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &script,
            ])
            .creation_flags(0x08000000)
            .output()
            .map_err(|_| "firewall_unavailable")?;
        if !out.status.success() {
            return Err("firewall_unavailable");
        }
        #[derive(serde::Deserialize)]
        struct Row {
            name: String,
            enabled: i32,
        }
        let rows: Vec<Row> =
            serde_json::from_slice(&out.stdout).map_err(|_| "firewall_unavailable")?;
        let mut profiles: Vec<Profile> = rows
            .into_iter()
            .map(|p| {
                Ok(Profile {
                    name: p.name,
                    enabled: match p.enabled {
                        0 => false,
                        1 => true,
                        _ => return Err("firewall_unavailable"),
                    },
                })
            })
            .collect::<Result<_>>()?;
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        if profiles.iter().map(|p| p.name.as_str()).collect::<Vec<_>>()
            != ["Domain", "Private", "Public"]
        {
            return Err("firewall_unavailable");
        }
        Ok(Status {
            platform: "windows",
            profiles,
        })
    }
    pub fn set(enabled: bool) -> Result<()> {
        let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let file = wide(
            system_dir()?
                .join("netsh.exe")
                .to_str()
                .ok_or("firewall_unavailable")?,
        );
        let verb = wide("runas");
        let params = wide(if enabled {
            "advfirewall set allprofiles state on"
        } else {
            "advfirewall set allprofiles state off"
        });
        let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOASYNC | SEE_MASK_NOCLOSEPROCESS;
        info.lpVerb = verb.as_ptr();
        info.lpFile = file.as_ptr();
        info.lpParameters = params.as_ptr();
        info.nShow = SW_HIDE;
        if unsafe { ShellExecuteExW(&mut info) } == 0 || info.hProcess.is_null() {
            return Err("firewall_change_failed");
        }
        let mut code = 1;
        let waited = unsafe { WaitForSingleObject(info.hProcess, INFINITE) };
        let read = unsafe { GetExitCodeProcess(info.hProcess, &mut code) };
        unsafe {
            CloseHandle(info.hProcess);
        }
        if waited == WAIT_OBJECT_0 && read != 0 && code == 0 {
            Ok(())
        } else {
            Err("firewall_change_failed")
        }
    }
}
#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    use super::*;
    pub fn status() -> Result<Status> {
        Err("firewall_unsupported")
    }
    pub fn set(_: bool) -> Result<()> {
        Err("firewall_unsupported")
    }
}
#[cfg(test)]
#[test]
fn disabling_requires_explicit_confirmation_before_any_os_access() {
    assert_eq!(
        set_enabled(false, false).unwrap_err(),
        "firewall_confirmation_required"
    );
}
