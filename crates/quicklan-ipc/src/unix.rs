use std::{
    io,
    os::{
        fd::AsRawFd,
        unix::{
            fs::PermissionsExt,
            net::{UnixListener, UnixStream},
        },
    },
    path::Path,
    time::{Duration, Instant},
};

pub type LocalStream = UnixStream;
pub fn disconnect(stream: &LocalStream) {
    let _ = stream.shutdown(std::net::Shutdown::Both);
}
pub fn is_elevated() -> bool {
    // Query only; the application never changes its own credentials.
    unsafe { libc::geteuid() == 0 }
}
pub struct LocalListener {
    socket: UnixListener,
    directory: tempfile::TempDir,
}
impl LocalListener {
    pub fn bind() -> io::Result<Self> {
        // tempdir uses atomic exclusive creation. Permissions precede socket bind.
        let directory = tempfile::Builder::new()
            .prefix("quicklan-")
            .tempdir_in("/tmp")?;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))?;
        let socket = UnixListener::bind(directory.path().join("control"))?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket, directory })
    }
    pub fn endpoint(&self) -> String {
        self.directory
            .path()
            .join("control")
            .to_string_lossy()
            .into_owned()
    }
    pub fn accept(&self, expected_pid: u32, timeout: Duration) -> io::Result<LocalStream> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.socket.accept() {
                Ok((stream, _)) => {
                    // PID comes from the OS elevation process handle/result, never the wire.
                    if peer_pid(&stream)? != expected_pid {
                        continue;
                    }
                    configure(&stream)?;
                    return Ok(stream);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(25))
                }
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Helper did not connect",
        ))
    }
}

pub fn connect(endpoint: &str, expected_parent_pid: u32) -> io::Result<LocalStream> {
    let path = Path::new(endpoint);
    if !path.is_absolute() || endpoint.len() > 100 {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let stream = UnixStream::connect(path)?;
    if peer_pid(&stream)? != expected_parent_pid {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    configure(&stream)?;
    Ok(stream)
}
fn configure(stream: &LocalStream) -> io::Result<()> {
    // macOS inherits O_NONBLOCK from the listening socket; Linux does not.
    // Framing uses bounded blocking reads on both platforms.
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(crate::IO_TIMEOUT))?;
    stream.set_write_timeout(Some(crate::IO_TIMEOUT))
}
pub fn peer_pid(stream: &LocalStream) -> io::Result<u32> {
    #[cfg(target_os = "macos")]
    {
        let mut pid: libc::pid_t = 0;
        let mut len = std::mem::size_of_val(&pid) as libc::socklen_t;
        // LOCAL_PEERPID is a kernel-reported credential, not client-provided data.
        let result = unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_LOCAL,
                libc::LOCAL_PEERPID,
                (&mut pid as *mut libc::pid_t).cast(),
                &mut len,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        if pid <= 0 {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(pid as u32)
    }
    #[cfg(target_os = "linux")]
    {
        let mut creds: libc::ucred = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of_val(&creds) as libc::socklen_t;
        let result = unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut creds as *mut libc::ucred).cast(),
                &mut len,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        if creds.pid <= 0 {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(creds.pid as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    #[test]
    fn accepted_stream_waits_for_delayed_frame_bytes() {
        let listener = LocalListener::bind().unwrap();
        let endpoint = listener.endpoint();
        let pid = std::process::id();
        let (accepted, wait_for_accept) = std::sync::mpsc::channel();
        let client = std::thread::spawn(move || {
            let mut stream = connect(&endpoint, pid).unwrap();
            // Keep the peer alive through PID authentication, even when a busy
            // runner schedules the listener after the original 100 ms delay.
            wait_for_accept
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
            std::thread::sleep(Duration::from_millis(100));
            stream.write_all(b"frame").unwrap();
            let mut ack = [0; 1];
            stream.read_exact(&mut ack).unwrap();
            assert_eq!(&ack, b"!");
        });
        let mut server = listener.accept(pid, Duration::from_secs(2)).unwrap();
        accepted.send(()).unwrap();
        let mut bytes = [0; 5];
        server.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"frame");
        server.write_all(b"!").unwrap();
        client.join().unwrap();
    }
    #[test]
    fn authenticates_os_pid_and_removes_owned_endpoint() {
        let listener = LocalListener::bind().unwrap();
        let endpoint = listener.endpoint();
        let other = endpoint.clone();
        let pid = std::process::id();
        let client = std::thread::spawn(move || connect(&other, pid).unwrap());
        let server = listener.accept(pid, Duration::from_secs(2)).unwrap();
        assert_eq!(peer_pid(&server).unwrap(), pid);
        client.join().unwrap();
        assert_eq!(
            std::fs::metadata(Path::new(&endpoint).parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        drop(server);
        drop(listener);
        assert!(!Path::new(&endpoint).exists());
    }
    #[test]
    fn rejects_a_server_with_the_wrong_os_pid() {
        let listener = LocalListener::bind().unwrap();
        assert!(connect(&listener.endpoint(), std::process::id() + 1).is_err());
    }
}
