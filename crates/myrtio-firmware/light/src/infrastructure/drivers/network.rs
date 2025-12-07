use embassy_net::{DhcpConfig, Runner, Stack, StackResources};
use esp_hal::peripherals::WIFI;
use esp_hal::rng::Rng;
use esp_radio::wifi::{Config as WifiConfig, WifiController, WifiDevice};
use static_cell::make_static;

pub(crate) fn init_network_stack(
    wifi_device: WIFI<'static>,
) -> (
    Stack<'static>,
    Runner<'static, WifiDevice<'static>>,
    WifiController<'static>,
) {
    let esp_radio_ctrl = &*make_static!(esp_radio::init().unwrap());
    let wifi_config = WifiConfig::default();
    let (controller, interfaces) =
        esp_radio::wifi::new(esp_radio_ctrl, wifi_device, wifi_config).unwrap();
    let dhcp_config = DhcpConfig::default();
    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let network_resources = make_static!(StackResources::<3>::new());
    let (stack, runner) =
        embassy_net::new(interfaces.sta, net_config, network_resources, get_seed());

    (stack, runner, controller)
}

fn get_seed() -> u64 {
    let rng = Rng::new();
    u64::from(rng.random()) << 32 | u64::from(rng.random())
}
