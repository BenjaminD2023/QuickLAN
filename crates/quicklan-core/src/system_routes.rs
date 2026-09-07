//! Read-only inspection before creating a virtual interface. No DNS/firewall writes.
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "android", windows, test))]
use crate::error::{Error, Result};
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "android", windows, test))]
use ipnet::Ipv4Net;

#[cfg(target_os = "macos")]
pub fn read() -> Result<Vec<Ipv4Net>> {
    let output = std::process::Command::new("/usr/sbin/netstat")
        .args(["-rn", "-f", "inet"])
        .env_clear()
        .env("LC_ALL", "C")
        .output()
        .map_err(|_| Error::CoreFailed)?;
    if !output.status.success() || output.stdout.len() > 2_097_152 {
        return Err(Error::CoreFailed);
    }
    parse_macos(std::str::from_utf8(&output.stdout).map_err(|_| Error::CoreFailed)?)
}

/// netstat abbreviates network addresses (e.g. 10/8, 192.168.1, 128.0/1).
/// Reject unknown rows instead of treating an unreadable table as conflict-free.
#[cfg(any(target_os = "macos", test))]
pub fn parse_macos(text: &str) -> Result<Vec<Ipv4Net>> {
    let mut routes = Vec::new();
    let mut header = false;
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line.starts_with("Destination ") {
            header = true;
            continue;
        }
        if !header {
            continue;
        }
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 4 {
            return Err(Error::CoreFailed);
        }
        if cols[0] == "default" {
            continue;
        }
        let (address, explicit) = cols[0]
            .split_once('/')
            .map_or((cols[0], None), |(a, p)| (a, Some(p)));
        let parts: Vec<_> = address.split('.').collect();
        if parts.is_empty() || parts.len() > 4 {
            return Err(Error::CoreFailed);
        }
        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part.parse().map_err(|_| Error::CoreFailed)?;
        }
        let prefix = match explicit {
            Some(p) => p.parse::<u8>().map_err(|_| Error::CoreFailed)?,
            None if cols[2].contains('H') => 32,
            None => (parts.len() * 8) as u8,
        };
        routes.push(Ipv4Net::new(octets.into(), prefix).map_err(|_| Error::CoreFailed)?);
    }
    if !header {
        return Err(Error::CoreFailed);
    }
    Ok(routes)
}

#[cfg(target_os = "linux")]
pub fn read() -> Result<Vec<Ipv4Net>> {
    let path = ["/usr/sbin/ip", "/sbin/ip", "/usr/bin/ip"]
        .into_iter()
        .find(|p| std::path::Path::new(p).is_file())
        .ok_or(Error::CoreFailed)?;
    let output = std::process::Command::new(path)
        .args(["-j", "-4", "route", "show", "table", "all"])
        .env_clear()
        .env("LC_ALL", "C")
        .output()
        .map_err(|_| Error::CoreFailed)?;
    if !output.status.success() || output.stdout.len() > 2_097_152 {
        return Err(Error::CoreFailed);
    }
    let rows: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).map_err(|_| Error::CoreFailed)?;
    let mut routes = Vec::new();
    for row in rows {
        let dst = row
            .get("dst")
            .and_then(|v| v.as_str())
            .ok_or(Error::CoreFailed)?;
        if dst == "default" {
            continue;
        }
        let value = if dst.contains('/') {
            dst.to_owned()
        } else {
            format!("{dst}/32")
        };
        routes.push(value.parse().map_err(|_| Error::CoreFailed)?);
    }
    Ok(routes)
}

#[cfg(windows)]
pub fn read() -> Result<Vec<Ipv4Net>> {
    use windows_sys::Win32::{
        NetworkManagement::IpHelper::{FreeMibTable, GetIpForwardTable2, MIB_IPFORWARD_TABLE2},
        Networking::WinSock::AF_INET,
    };
    let mut table: *mut MIB_IPFORWARD_TABLE2 = std::ptr::null_mut();
    if unsafe { GetIpForwardTable2(AF_INET, &mut table) } != 0 || table.is_null() {
        return Err(Error::CoreFailed);
    }
    struct OwnedTable(*mut MIB_IPFORWARD_TABLE2);
    impl Drop for OwnedTable {
        fn drop(&mut self) {
            unsafe {
                FreeMibTable(self.0.cast());
            }
        }
    }
    let table = OwnedTable(table);
    let count = unsafe { (*table.0).NumEntries as usize };
    if count > 65536 {
        return Err(Error::CoreFailed);
    }
    let rows = unsafe { std::slice::from_raw_parts((*table.0).Table.as_ptr(), count) };
    rows.iter()
        .map(|row| {
            let raw = unsafe { row.DestinationPrefix.Prefix.Ipv4.sin_addr.S_un.S_addr };
            Ipv4Net::new(
                std::net::Ipv4Addr::from(u32::from_be(raw)),
                row.DestinationPrefix.PrefixLength,
            )
            .map_err(|_| Error::CoreFailed)
        })
        .collect()
}

#[cfg(target_os = "android")]
pub fn read() -> Result<Vec<Ipv4Net>> {
    parse_proc_net_route(
        &std::fs::read_to_string("/proc/net/route").map_err(|_| Error::CoreFailed)?,
    )
}

/// Destination and mask are little-endian hex, as in /proc/net/route.
#[cfg(any(target_os = "android", test))]
pub fn parse_proc_net_route(text: &str) -> Result<Vec<Ipv4Net>> {
    let mut routes = Vec::new();
    let mut header = false;
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if !header {
            header = line.contains("Destination") && line.contains("Mask");
            continue;
        }
        let mut cols = line.split_whitespace();
        let _iface = cols.next().ok_or(Error::CoreFailed)?;
        let destination = cols.next().ok_or(Error::CoreFailed)?;
        let _gateway = cols.next().ok_or(Error::CoreFailed)?;
        let _flags = cols.next().ok_or(Error::CoreFailed)?;
        let _refcnt = cols.next().ok_or(Error::CoreFailed)?;
        let _use = cols.next().ok_or(Error::CoreFailed)?;
        let _metric = cols.next().ok_or(Error::CoreFailed)?;
        let mask = cols.next().ok_or(Error::CoreFailed)?;
        let dest = u32::from_be(
            u32::from_str_radix(destination, 16).map_err(|_| Error::CoreFailed)?,
        );
        let mask = u32::from_be(u32::from_str_radix(mask, 16).map_err(|_| Error::CoreFailed)?);
        if mask == 0 {
            continue;
        }
        let prefix = mask.count_ones() as u8;
        if prefix == 0 || prefix > 32 || mask != u32::MAX << (32 - prefix) {
            return Err(Error::CoreFailed);
        }
        let net = Ipv4Net::new(std::net::Ipv4Addr::from(dest), prefix)
            .map_err(|_| Error::CoreFailed)?;
        routes.push(net);
    }
    if !header {
        return Err(Error::CoreFailed);
    }
    Ok(routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn abbreviated_macos_routes_include_split_default_vpns() {
        let text = "Routing tables\nInternet:\nDestination Gateway Flags Netif\ndefault 192.168.1.1 UG en0\n10/8 link#4 UCS en0\n192.168.1 link#4 UCS en0\n128.0/1 198.18.0.1 UG utun7\n192.168.1.7 00:11:22:33:44:55 UHL en0\n";
        let rows = parse_macos(text).unwrap();
        assert_eq!(
            rows.iter().map(ToString::to_string).collect::<Vec<_>>(),
            [
                "10.0.0.0/8",
                "192.168.1.0/24",
                "128.0.0.0/1",
                "192.168.1.7/32"
            ]
        );
        assert_eq!(
            crate::routes::check_conflicts("10.73.42.0/24", &rows, &[]),
            Err(Error::RouteConflict)
        );
        assert!(parse_macos("unexpected output").is_err());
        assert!(parse_macos("Destination Gateway Flags Netif\n10/99 x U x").is_err());
    }
    #[test]
    fn proc_net_route_reads_private_lan_and_skips_default() {
        let text = "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\tMask\tMTU\tWindow\tIRTT\nwlan0\t00000000\t0100A8C0\t0003\t0\t0\t0\t00000000\t0\t0\t0\nwlan0\t0001A8C0\t00000000\t0001\t0\t0\t0\t00FFFFFF\t0\t0\t0\n";
        let rows = parse_proc_net_route(text).unwrap();
        assert_eq!(
            rows.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["192.168.1.0/24"]
        );
        assert_eq!(
            crate::routes::check_conflicts("192.168.1.0/24", &rows, &[]),
            Err(Error::RouteConflict)
        );
        assert!(crate::routes::check_conflicts("10.73.42.0/24", &rows, &[]).is_ok());
    }
}
