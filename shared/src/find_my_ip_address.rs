use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// Given an IPv4 Address, attempt to find an available port on the current host
pub fn find_my_ip_address() -> Option<IpAddr> {
    let ip = local_ipaddress::get().unwrap_or_default();

    if let Ok(addr) = ip.parse::<Ipv4Addr>() {
        return Some(IpAddr::V4(addr));
    } else if let Ok(addr) = ip.parse::<Ipv6Addr>() {
        return Some(IpAddr::V6(addr));
    } else {
        return None;
    }
}
