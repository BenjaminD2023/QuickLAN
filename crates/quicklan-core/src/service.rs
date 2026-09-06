//! One explicit application-port check; no scans, ICMP inference or firewall writes.
use serde::Serialize;
use std::{
    net::{SocketAddr, SocketAddrV4, TcpStream},
    time::Duration,
};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeResult {
    Reachable,
    Refused,
    TimedOut,
    Unreachable,
}

/// Caller must select the endpoint from the active, validated peer snapshot.
pub fn probe(endpoint: SocketAddrV4) -> ProbeResult {
    match TcpStream::connect_timeout(&SocketAddr::V4(endpoint), Duration::from_secs(3)) {
        Ok(stream) => {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            ProbeResult::Reachable
        }
        Err(error) => match error.kind() {
            std::io::ErrorKind::ConnectionRefused => ProbeResult::Refused,
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => ProbeResult::TimedOut,
            _ => ProbeResult::Unreachable,
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinguishes_real_listening_and_closed_tcp_ports() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let SocketAddr::V4(endpoint) = listener.local_addr().unwrap() else {
            panic!()
        };
        assert_eq!(probe(endpoint), ProbeResult::Reachable);
        drop(listener);
        assert_eq!(probe(endpoint), ProbeResult::Refused);
    }
}
