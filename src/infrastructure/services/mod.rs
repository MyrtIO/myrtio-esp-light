pub(crate) mod dhcp;
pub mod http;
pub(crate) mod light_state;
pub(crate) mod ota;
pub(crate) mod persistence;


pub use dhcp::{
    DHCP_ACK, DHCP_DISCOVER, DHCP_OFFER, DHCP_REQUEST, DhcpRequest, allocate_ip,
    build_dhcp_response, parse_dhcp_request,
};
pub use light_state::LightStateService;
pub use persistence::{LightStatePersistenceService, get_persistence_receiver};
pub(crate) use ota::{update_from_http, OtaError};