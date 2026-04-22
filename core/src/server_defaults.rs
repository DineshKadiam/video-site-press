// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::{IpAddr, Ipv4Addr};
use std::ops::Range;

pub const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// The default port used by hyper8 is 49737 ("Dial 127.0.0.1:HYPER"),
/// but in case this isn't available we try to fall back to a port
/// on one of two ranges (16 ports from 49737 and 8080 respectively).
const DEFAULT_PORT_RANGES: [Range<u16>; 2] = [
    49737..49753,
    8080..8096
];

pub fn default_ports_iter() -> impl Iterator<Item = u16> {
    DEFAULT_PORT_RANGES
        .iter()
        .map(|port_range| port_range.clone())
        .flatten()
}

pub fn default_ports_display() -> String {
    DEFAULT_PORT_RANGES
        .map(|port_range| {
            let start = port_range.start;
            let end = port_range.end;

            format!("{start}-{end}")
        })
        .join(", ")
}
