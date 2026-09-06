//! OS-authenticated, local-only duplex transport. No network listener.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

pub const IO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
