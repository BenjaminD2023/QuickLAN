use quicklan_core::error::{Error, Result};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub struct ElevatedChild {
    pub pid: u32,
    #[cfg(windows)]
    handle: std::os::windows::io::OwnedHandle,
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
pub fn launch(engine: &Path, endpoint: &str, parent: u32) -> Result<ElevatedChild> {
    let engine = engine.to_str().ok_or(Error::UnsafePath)?;
    // Only fixed flags and generated endpoint/PID reach the system prompt.
    // Credentials are sent later, over the OS-authenticated socket.
    let command = format!("/usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin HOME=/var/empty {} --ipc {} --parent-pid {} </dev/null >/dev/null 2>&1 & /bin/echo $!", shell_quote(engine), shell_quote(endpoint), parent);
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
pub fn launch(engine: &Path, endpoint: &str, parent: u32) -> Result<ElevatedChild> {
    use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
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
    Ok(ElevatedChild { pid, handle })
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn launch(_: &Path, _: &str, _: u32) -> Result<ElevatedChild> {
    Err(Error::HelperUnavailable)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
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
