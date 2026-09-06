use std::{
    fs::File,
    io::{self, Read, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        LocalFree, ERROR_NO_DATA, ERROR_PIPE_CONNECTED, ERROR_PIPE_LISTENING, GENERIC_READ,
        GENERIC_WRITE, INVALID_HANDLE_VALUE,
    },
    Security::{
        Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1},
        SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::{
        CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
    },
    System::Pipes::*,
};

fn wide(value: &str) -> Vec<u16> {
    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(Some(0))
        .collect()
}
fn denied() -> io::Error {
    io::ErrorKind::PermissionDenied.into()
}
fn configure(file: &File) -> io::Result<()> {
    let mode = PIPE_READMODE_BYTE | PIPE_NOWAIT;
    if unsafe {
        SetNamedPipeHandleState(
            file.as_raw_handle(),
            &mode,
            std::ptr::null(),
            std::ptr::null(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub struct LocalStream {
    file: File,
    server: bool,
}
pub fn disconnect(stream: &LocalStream) {
    if stream.server {
        unsafe {
            DisconnectNamedPipe(stream.file.as_raw_handle());
        }
    }
}
impl LocalStream {
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            file: self.file.try_clone()?,
            server: self.server,
        })
    }
}
impl Read for LocalStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let deadline = Instant::now() + crate::IO_TIMEOUT;
        loop {
            let mut available = 0;
            if unsafe {
                PeekNamedPipe(
                    self.file.as_raw_handle(),
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    &mut available,
                    std::ptr::null_mut(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            if available > 0 {
                let len = buffer.len().min(available as usize);
                return self.file.read(&mut buffer[..len]);
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Write for LocalStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let deadline = Instant::now() + crate::IO_TIMEOUT;
        loop {
            match self.file.write(buffer) {
                Ok(0) => (),
                other => return other,
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    // FlushFileBuffers waits for an untrusted reader without a bound. Framing is
    // already written synchronously into the kernel pipe; no disk flush is needed.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct LocalListener {
    file: File,
    endpoint: String,
    _directory: tempfile::TempDir,
}
impl LocalListener {
    pub fn bind() -> io::Result<Self> {
        let directory = tempfile::Builder::new().prefix("quicklan-").tempdir()?;
        let endpoint = format!(
            r"\\.\pipe\QuickLAN-{}-{}",
            std::process::id(),
            directory.path().file_name().unwrap().to_string_lossy()
        );
        // Protected DACL: object owner, SYSTEM, administrators. The kernel PID
        // check below further restricts this to the exact elevated child process.
        let sddl = wide("D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;OW)");
        let mut descriptor = std::ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let security = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let handle = unsafe {
            CreateNamedPipeW(
                wide(&endpoint).as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                65536,
                65536,
                0,
                &security,
            )
        };
        let error = io::Error::last_os_error();
        unsafe {
            LocalFree(descriptor);
        }
        if handle == INVALID_HANDLE_VALUE {
            return Err(error);
        }
        Ok(Self {
            file: unsafe { File::from_raw_handle(handle) },
            endpoint,
            _directory: directory,
        })
    }
    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }
    pub fn accept(&self, expected_pid: u32, timeout: Duration) -> io::Result<LocalStream> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let result =
                unsafe { ConnectNamedPipe(self.file.as_raw_handle(), std::ptr::null_mut()) };
            let code = io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
            if result != 0 || code == ERROR_PIPE_CONNECTED {
                let stream = LocalStream {
                    file: self.file.try_clone()?,
                    server: true,
                };
                if peer_pid(&stream)? == expected_pid {
                    return Ok(stream);
                }
                unsafe {
                    DisconnectNamedPipe(self.file.as_raw_handle());
                }
            } else if code != ERROR_PIPE_LISTENING && code != ERROR_NO_DATA {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        Err(io::ErrorKind::TimedOut.into())
    }
}
pub fn connect(endpoint: &str, expected_parent_pid: u32) -> io::Result<LocalStream> {
    if !endpoint.starts_with(r"\\.\pipe\QuickLAN-")
        || endpoint.len() > 200
        || endpoint.as_bytes().contains(&0)
    {
        return Err(denied());
    }
    let handle = unsafe {
        CreateFileW(
            wide(endpoint).as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let stream = LocalStream {
        file: unsafe { File::from_raw_handle(handle) },
        server: false,
    };
    if peer_pid(&stream)? != expected_parent_pid {
        return Err(denied());
    }
    configure(&stream.file)?;
    Ok(stream)
}
pub fn peer_pid(stream: &LocalStream) -> io::Result<u32> {
    let mut pid = 0;
    let ok = unsafe {
        if stream.server {
            GetNamedPipeClientProcessId(stream.file.as_raw_handle(), &mut pid)
        } else {
            GetNamedPipeServerProcessId(stream.file.as_raw_handle(), &mut pid)
        }
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    if pid == 0 {
        return Err(denied());
    }
    Ok(pid)
}

pub fn is_elevated() -> bool {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    let mut token = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut returned = 0;
    let ok = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
    };
    unsafe {
        CloseHandle(token);
    }
    ok != 0 && elevation.TokenIsElevated != 0
}
