use embassy_executor::Spawner;
use embassy_net::{IpAddress, Stack, dns::DnsQueryType, tcp::TcpSocket};
use embassy_sync::channel::Channel;
use embassy_time::Duration;
#[cfg(feature = "log")]
use esp_println::println;
use heapless::String;
use myrtio_mqtt::{
    client::{MqttClient, MqttOptions},
    runtime::{MqttModule, MqttRuntime, PublishRequestChannel},
    transport::TcpTransport,
};

use crate::{
    config::{MqttConfig, hardware_id},
    mk_static,
};

const MQTT_OUTBOX_DEPTH: usize = 4;
const MQTT_MAX_TOPICS: usize = 8;
const MQTT_BUF_SIZE: usize = 2048;

static PUBLISH_CHANNEL: PublishRequestChannel<'static, MQTT_OUTBOX_DEPTH> =
    Channel::new();

pub fn start_mqtt_client(
    spawner: Spawner,
    stack: Stack<'static>,
    module: &'static mut dyn MqttModule,
    mqtt_config: MqttConfig,
) {
    spawner
        .spawn(mqtt_client_task(stack, module, mqtt_config))
        .ok();
}

/// MQTT runtime task that accepts any module implementing `MqttModule`.
#[embassy_executor::task]
pub async fn mqtt_client_task(
    stack: Stack<'static>,
    module: &'static mut dyn MqttModule,
    mqtt_config: MqttConfig,
) {
    #[cfg(feature = "log")]
    println!("mqtt: starting runtime task");
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let device_id = mk_static!(String<32>, format_device_id(hardware_id()));
    #[cfg(feature = "log")]
    println!("mqtt: device id: {}", device_id);
    loop {
        if let Err(_e) = run_mqtt_client(
            stack,
            module,
            &mqtt_config,
            device_id,
            &mut rx_buffer,
            &mut tx_buffer,
        )
        .await
        {
            #[cfg(feature = "log")]
            println!("mqtt: connection lost, reconnecting in 2s...");
            embassy_time::Timer::after(Duration::from_secs(2)).await;
        }
    }
}

fn format_device_id(hardware_id: u32) -> String<32> {
    use core::fmt::Write;
    let mut device_id = String::<32>::new();
    let _ = write!(device_id, "myrtio-light-{:04X}", hardware_id);
    device_id
}

async fn run_mqtt_client(
    stack: Stack<'static>,
    module: &mut dyn MqttModule,
    mqtt_config: &MqttConfig,
    device_id: &'static str,
    rx_buffer: &mut [u8],
    tx_buffer: &mut [u8],
) -> Result<(), ()> {
    let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(60)));

    let broker_addr = resolve_host(stack, mqtt_config.host.as_str()).await?;

    #[cfg(feature = "log")]
    println!(
        "mqtt: connecting to broker {:?}:{}...",
        broker_addr, mqtt_config.port
    );

    let connection_result = socket.connect((broker_addr, mqtt_config.port)).await;
    if let Err(_e) = connection_result {
        socket.abort();
        #[cfg(feature = "log")]
        println!("mqtt: TCP connect failed: {:?}", _e);
        return Err(());
    }
    #[cfg(feature = "log")]
    println!("mqtt: TCP socket connected");

    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    let options =
        MqttOptions::new(device_id).with_keep_alive(Duration::from_secs(15));
    let mqtt: MqttClient<_, MQTT_MAX_TOPICS, MQTT_BUF_SIZE> =
        MqttClient::new(transport, options);

    // Create the runtime with the provided module
    let mut runtime: MqttRuntime<
        '_,
        _,
        &mut dyn MqttModule,
        MQTT_MAX_TOPICS,
        MQTT_BUF_SIZE,
        MQTT_OUTBOX_DEPTH,
    > = MqttRuntime::new(mqtt, module, PUBLISH_CHANNEL.receiver());

    runtime.run().await.map_err(|_e| {
        #[cfg(feature = "log")]
        println!("mqtt: runtime error: {:?}", _e);
    })
}

/// Resolves a hostname to an IP address
async fn resolve_host(stack: Stack<'static>, host: &str) -> Result<IpAddress, ()> {
    if let Ok(ip) = host.parse::<embassy_net::Ipv4Address>() {
        return Ok(IpAddress::Ipv4(ip));
    }

    let Ok(addresses) = stack.dns_query(host, DnsQueryType::A).await else {
        return Err(());
    };

    addresses.first().copied().ok_or(())
}
