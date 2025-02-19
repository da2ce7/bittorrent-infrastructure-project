use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};

use chrono::{DateTime, Duration, Utc};

use crate::bt::{self, NodeId};
use crate::net::IpAddr;

/// Allows us to time travel into the future.
///
/// # Panics
///
/// Panics if offset is out of range.
#[must_use]
pub fn travel_into_future(offset: Duration) -> DateTime<Utc> {
    Utc::now().checked_add_signed(offset).unwrap()
}

/// Allows us to time travel into the past.
///
/// # Panics
///
/// Panics if offset is out of range.
#[must_use]
pub fn travel_into_past(offset: Duration) -> DateTime<Utc> {
    Utc::now().checked_sub_signed(offset).unwrap()
}

/// Generates a dummy Ipv4 address as an `IpAddr`.
#[must_use]
pub fn dummy_ipv4_addr() -> IpAddr {
    let v4_addr = Ipv4Addr::new(127, 0, 0, 1);

    IpAddr::V4(v4_addr)
}

/// Generates a dummy ipv6 address as an `IpAddr`.
#[must_use]
pub fn dummy_ipv6_addr() -> IpAddr {
    let v6_addr = Ipv6Addr::new(127, 0, 0, 1, 0, 0, 0, 0);

    IpAddr::V6(v6_addr)
}

/// Generates a dummy socket address v4 as a `SocketAddr`.
#[must_use]
pub fn dummy_socket_addr_v4() -> SocketAddr {
    let v4_addr = Ipv4Addr::new(127, 0, 0, 1);
    let v4_socket = SocketAddrV4::new(v4_addr, 0);

    SocketAddr::V4(v4_socket)
}

/// Generates a block of unique ipv4 addresses as Vec<`SocketAddr`>
#[must_use]
pub fn dummy_block_socket_addrs(num_addrs: u16) -> Vec<SocketAddr> {
    let mut addr_block = Vec::with_capacity(num_addrs as usize);

    for port in 0..num_addrs {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let sock_addr = SocketAddrV4::new(ip, port);

        addr_block.push(SocketAddr::V4(sock_addr));
    }

    addr_block
}

/// Generates a dummy node id as a `NodeId`.
#[must_use]
pub fn dummy_node_id() -> NodeId {
    NodeId::from([0u8; bt::NODE_ID_LEN])
}

/// Generates a block of unique dummy node ids as Vec<`NodeId`>
#[must_use]
pub fn dummy_block_node_ids(num_ids: u8) -> Vec<NodeId> {
    let mut id_block = Vec::with_capacity(num_ids as usize);

    for repeat in 0..num_ids {
        let mut id = [0u8; bt::NODE_ID_LEN];

        for byte in &mut id {
            *byte = repeat;
        }

        id_block.push(id.into());
    }

    id_block
}
