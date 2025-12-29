//! DHCP Protocol Implementation
//!
//! Provides DHCP message parsing and response building for a simple stateless
//! server.

use embassy_net::Ipv4Address;

/// DHCP message types
pub(crate) const DHCP_DISCOVER: u8 = 1;
pub(crate) const DHCP_OFFER: u8 = 2;
pub(crate) const DHCP_REQUEST: u8 = 3;
pub(crate) const DHCP_ACK: u8 = 5;

/// DHCP options
const DHCP_OPTION_MESSAGE_TYPE: u8 = 53;
const DHCP_OPTION_SERVER_ID: u8 = 54;
const DHCP_OPTION_LEASE_TIME: u8 = 51;
const DHCP_OPTION_SUBNET_MASK: u8 = 1;
const DHCP_OPTION_ROUTER: u8 = 3;
const DHCP_OPTION_DNS: u8 = 6;
const DHCP_OPTION_END: u8 = 255;

/// DHCP magic cookie
const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];

/// Lease configuration
const LEASE_TIME_SECS: u32 = 3600; // 1 hour
const SUBNET_MASK: Ipv4Address = Ipv4Address::new(255, 255, 255, 0);

/// Minimum DHCP packet size (BOOTP header + magic cookie)
const MIN_DHCP_PACKET_SIZE: usize = 240;

/// Parsed DHCP request
#[derive(Debug)]
pub(crate) struct DhcpRequest {
    /// Transaction ID
    pub xid: [u8; 4],
    /// Client MAC address
    pub client_mac: [u8; 6],
    /// Message type (DISCOVER, REQUEST, etc.)
    pub message_type: u8,
}

/// Parse a DHCP request from a raw packet
///
/// Returns `None` if the packet is invalid or not a BOOTREQUEST
pub(crate) fn parse_dhcp_request(packet: &[u8]) -> Option<DhcpRequest> {
    if packet.len() < MIN_DHCP_PACKET_SIZE {
        return None;
    }

    // Check op code (must be BOOTREQUEST = 1)
    if packet[0] != 1 {
        return None;
    }

    // Get transaction ID
    let mut xid = [0u8; 4];
    xid.copy_from_slice(&packet[4..8]);

    // Get client MAC address
    let mut client_mac = [0u8; 6];
    client_mac.copy_from_slice(&packet[28..34]);

    // Check magic cookie
    if packet[236..240] != DHCP_MAGIC_COOKIE {
        return None;
    }

    // Find message type in options
    let options = &packet[240..];
    let message_type = find_dhcp_option(options, DHCP_OPTION_MESSAGE_TYPE)
        .and_then(|data| data.first().copied())?;

    Some(DhcpRequest {
        xid,
        client_mac,
        message_type,
    })
}

/// Allocate an IP address for a client based on their MAC address
///
/// Uses a simple stateless algorithm to derive a consistent IP from the MAC.
/// Returns an address in the range 192.168.4.2 - 192.168.4.50
pub(crate) fn allocate_ip(mac: &[u8; 6]) -> Ipv4Address {
    let offset = (mac[5] % 49) + 2;
    Ipv4Address::new(192, 168, 4, offset)
}

/// Build a DHCP response (OFFER or ACK)
///
/// Returns the length of the response packet
pub(crate) fn build_dhcp_response(
    ap_ip_address: Ipv4Address,
    buffer: &mut [u8],
    request: &DhcpRequest,
    offered_ip: Ipv4Address,
    response_type: u8,
) -> usize {
    buffer.fill(0);

    // BOOTP header
    buffer[0] = 2; // op: BOOTREPLY
    buffer[1] = 1; // htype: Ethernet
    buffer[2] = 6; // hlen: MAC length
    buffer[3] = 0; // hops

    // Transaction ID
    buffer[4..8].copy_from_slice(&request.xid);

    // secs, flags
    buffer[8..10].copy_from_slice(&[0, 0]);
    buffer[10..12].copy_from_slice(&[0x80, 0x00]); // Broadcast flag

    // ciaddr (client IP) - 0
    // yiaddr (your IP) - offered IP
    buffer[16..20].copy_from_slice(&offered_ip.octets());

    // siaddr (server IP)
    buffer[20..24].copy_from_slice(&ap_ip_address.octets());

    // giaddr (gateway IP) - 0

    // chaddr (client hardware address)
    buffer[28..34].copy_from_slice(&request.client_mac);

    // sname, file - leave as 0

    // DHCP magic cookie at offset 236
    buffer[236..240].copy_from_slice(&DHCP_MAGIC_COOKIE);

    // DHCP options start at 240
    let mut opt_idx = 240;

    // Message type
    buffer[opt_idx] = DHCP_OPTION_MESSAGE_TYPE;
    buffer[opt_idx + 1] = 1;
    buffer[opt_idx + 2] = response_type;
    opt_idx += 3;

    // Server identifier
    buffer[opt_idx] = DHCP_OPTION_SERVER_ID;
    buffer[opt_idx + 1] = 4;
    buffer[opt_idx + 2..opt_idx + 6].copy_from_slice(&ap_ip_address.octets());
    opt_idx += 6;

    // Lease time
    buffer[opt_idx] = DHCP_OPTION_LEASE_TIME;
    buffer[opt_idx + 1] = 4;
    buffer[opt_idx + 2..opt_idx + 6].copy_from_slice(&LEASE_TIME_SECS.to_be_bytes());
    opt_idx += 6;

    // Subnet mask
    buffer[opt_idx] = DHCP_OPTION_SUBNET_MASK;
    buffer[opt_idx + 1] = 4;
    buffer[opt_idx + 2..opt_idx + 6].copy_from_slice(&SUBNET_MASK.octets());
    opt_idx += 6;

    // Router (gateway)
    buffer[opt_idx] = DHCP_OPTION_ROUTER;
    buffer[opt_idx + 1] = 4;
    buffer[opt_idx + 2..opt_idx + 6].copy_from_slice(&ap_ip_address.octets());
    opt_idx += 6;

    // DNS server (use AP as DNS too for captive portal)
    buffer[opt_idx] = DHCP_OPTION_DNS;
    buffer[opt_idx + 1] = 4;
    buffer[opt_idx + 2..opt_idx + 6].copy_from_slice(&ap_ip_address.octets());
    opt_idx += 6;

    // End option
    buffer[opt_idx] = DHCP_OPTION_END;
    opt_idx += 1;

    opt_idx
}

/// Find a DHCP option in the options section
///
/// The options slice should start AFTER the magic cookie (at offset 240 in the
/// packet)
fn find_dhcp_option(options: &[u8], option_code: u8) -> Option<&[u8]> {
    let mut i = 0;

    while i < options.len() {
        let code = options[i];
        if code == DHCP_OPTION_END {
            break;
        }
        if code == 0 {
            // Padding
            i += 1;
            continue;
        }
        if i + 1 >= options.len() {
            break;
        }
        let len = options[i + 1] as usize;
        if i + 2 + len > options.len() {
            break;
        }
        if code == option_code {
            return Some(&options[i + 2..i + 2 + len]);
        }
        i += 2 + len;
    }
    None
}
