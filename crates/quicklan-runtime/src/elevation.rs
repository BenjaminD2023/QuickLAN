use quicklan_core::error::{Error, Result};
#[cfg(unix)]
use std::time::Instant;
use std::{path::Path, time::Duration};

pub struct ElevatedChild {
    pub pid: u32,
    #[cfg(windows)]
    handle: std::os::windows::io::OwnedHandle,
    #[cfg(windows)]
    _held_files: Vec<std::fs::File>,
}
impl ElevatedChild {
    pub fn wait(&self, timeout: Duration) -> bool {
        #[cfg(unix)]
        {
            let deadline = Instant::now() + timeout;
            while Instant::now() < deadline {
                let result = unsafe { libc::kill(self.pid as libc::pid_t, 0) };
                if result != 0
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            false
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            unsafe {
                windows_sys::Win32::System::Threading::WaitForSingleObject(
                    self.handle.as_raw_handle(),
                    timeout.as_millis().min(u32::MAX as u128) as u32,
                ) == 0
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
#[cfg(target_os = "macos")]
fn apple_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "macos")]
fn launch_script(engine: &str, digest: &str, endpoint: &str, parent: u32) -> Result<String> {
    if !quicklan_core::model::valid_hex(digest, 64) {
        return Err(Error::IncompatibleCore);
    }
    // Copy before verifying, inside a root-owned exclusive directory. Executing
    // that verified copy closes the user-owned bundle's check/launch race.
    // The root supervisor removes only its own directory after the child exits.
    Ok(format!(
        r#"set -eu
umask 077
stage=$(/usr/bin/mktemp -d /private/var/tmp/quicklan-engine.XXXXXXXX)
trap '/bin/rm -rf -- "$stage"' EXIT
/bin/cp -- {} "$stage/quicklan-engine"
actual=$(/usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin /usr/bin/shasum -a 256 "$stage/quicklan-engine")
case "$actual" in '{}  '*) ;; *) exit 2 ;; esac
/bin/chmod 500 "$stage/quicklan-engine"
(
  trap '/bin/rm -rf -- "$stage"' EXIT
  /usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin HOME=/var/empty "$stage/quicklan-engine" --ipc {} --parent-pid {} </dev/null >/dev/null 2>&1 &
  child=$!
  /usr/bin/printf '%s\n' "$child" > "$stage/pid"
  wait "$child" || true
) </dev/null >/dev/null 2>&1 &
guard=$!
attempt=0
while [ ! -f "$stage/pid" ]; do
  attempt=$((attempt + 1))
  [ "$attempt" -lt 100 ] || exit 2
  /bin/sleep 0.05
done
/bin/cat "$stage/pid"
trap - EXIT
"#,
        shell_quote(engine),
        digest,
        shell_quote(endpoint),
        parent
    ))
}

#[cfg(target_os = "macos")]
pub fn launch(engine: &Path, digest: &str, endpoint: &str, parent: u32) -> Result<ElevatedChild> {
    let engine = engine.to_str().ok_or(Error::UnsafePath)?;
    // Only fixed flags and generated endpoint/PID reach the system prompt.
    // Credentials are sent later, over the OS-authenticated socket.
    let command = launch_script(engine, digest, endpoint, parent)?;
    let script = format!(
        "do shell script {} with administrator privileges",
        apple_quote(&command)
    );
    let output = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .map_err(|_| Error::PermissionDenied)?;
    if !output.status.success() || output.stdout.len() > 32 {
        return Err(Error::PermissionDenied);
    }
    let pid = std::str::from_utf8(&output.stdout)
        .map_err(|_| Error::CoreFailed)?
        .trim()
        .parse::<u32>()
        .map_err(|_| Error::CoreFailed)?;
    if pid <= 1 || pid == parent {
        return Err(Error::Unauthorized);
    }
    Ok(ElevatedChild { pid })
}

#[cfg(windows)]
pub fn launch(engine: &Path, digest: &str, endpoint: &str, parent: u32) -> Result<ElevatedChild> {
    use std::io::Read;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
    // Deny replacement and writes to both executable and driver during launch.
    // Retain directory handles too so a writable ancestor cannot be renamed.
    let mut held = Vec::new();
    for directory in engine.ancestors().skip(1) {
        if std::fs::symlink_metadata(directory)
            .map_err(|_| Error::UnsafePath)?
            .file_attributes()
            & 0x400
            != 0
        {
            return Err(Error::UnsafePath);
        }
        held.push(
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(1)
                .custom_flags(0x02000000)
                .open(directory)
                .map_err(|_| Error::UnsafePath)?,
        );
    }
    for (path, expected) in [
        (engine.to_path_buf(), digest),
        (
            engine.with_file_name("wintun.dll"),
            "e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce",
        ),
    ] {
        if std::fs::symlink_metadata(&path)
            .map_err(|_| Error::UnsafePath)?
            .file_attributes()
            & 0x400
            != 0
        {
            return Err(Error::UnsafePath);
        }
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .map_err(|_| Error::UnsafePath)?;
        let mut bytes = Vec::new();
        (&mut file)
            .take(150_000_001)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::IncompatibleCore)?;
        quicklan_core::adapter::verify_bytes(&bytes, expected)?;
        held.push(file);
    }
    use windows_sys::Win32::{
        System::Threading::GetProcessId,
        UI::{
            Shell::{
                ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
            },
            WindowsAndMessaging::SW_HIDE,
        },
    };
    fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
        s.encode_wide().chain(Some(0)).collect()
    }
    let file = wide(engine.as_os_str());
    let directory = wide(engine.parent().ok_or(Error::UnsafePath)?.as_os_str());
    let verb = wide(std::ffi::OsStr::new("runas"));
    let parameters = wide(std::ffi::OsStr::new(&format!(
        "--ipc \"{endpoint}\" --parent-pid {parent}"
    )));
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
    info.lpVerb = verb.as_ptr();
    info.lpFile = file.as_ptr();
    info.lpParameters = parameters.as_ptr();
    info.lpDirectory = directory.as_ptr();
    info.nShow = SW_HIDE;
    if unsafe { ShellExecuteExW(&mut info) } == 0 || info.hProcess.is_null() {
        return Err(Error::PermissionDenied);
    }
    let handle = unsafe { std::os::windows::io::OwnedHandle::from_raw_handle(info.hProcess) };
    let pid = unsafe { GetProcessId(info.hProcess) };
    if pid == 0 || pid == parent {
        return Err(Error::Unauthorized);
    }
    Ok(ElevatedChild {
        pid,
        handle,
        _held_files: held,
    })
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn launch(_: &Path, _: &str, _: &str, _: u32) -> Result<ElevatedChild> {
    Err(Error::HelperUnavailable)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn root_staging_script_rejects_changed_payload_before_execution() {
        let script = launch_script(
            "/bin/echo",
            &"0".repeat(64),
            "/tmp/unused",
            std::process::id(),
        )
        .unwrap();
        let output = std::process::Command::new("/bin/sh")
            .args(["-c", &script])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(launch_script("/bin/echo", "$(touch unsafe)", "/tmp/unused", 42).is_err());
    }
    #[test]
    fn shell_metacharacters_remain_literal_arguments() {
        let value = "a'b\" $HOME `touch x` \\";
        let output = std::process::Command::new("/bin/sh")
            .args(["-c", &format!("printf '%s' {}", shell_quote(value))])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(output.stdout).unwrap(), value);
        assert_eq!(apple_quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }
}
