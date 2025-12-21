//! MQTT Runtime Task
//!
//! This module provides the infrastructure-level MQTT task that accepts any
//! `MqttModule` implementation via a trait object.

use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_println::println;
use heapless::String;
use myrtio_mqtt::runtime::{MqttModule, MqttRuntime, PublishRequestChannel};
use myrtio_mqtt::{
    client::{MqttClient, MqttOptions},
    transport::TcpTransport,
};

use crate::config::{MqttConfig, hardware_id};
use crate::infrastructure::drivers::resolve_host;
use crate::mk_static;

const MQTT_OUTBOX_DEPTH: usize = 4;
const MQTT_MAX_TOPICS: usize = 8;
const MQTT_BUF_SIZE: usize = 1024;

static PUBLISH_CHANNEL: PublishRequestChannel<'static, MQTT_OUTBOX_DEPTH> = Channel::new();

/// MQTT runtime task that accepts any module implementing `MqttModule`.
#[embassy_executor::task]
pub async fn mqtt_runtime_task(
    stack: Stack<'static>,
    module: &'static mut dyn MqttModule,
    mqtt_config: MqttConfig,
) {
    println!("mqtt: starting runtime task");
    let device_id = mk_static!(String<17>, format_device_id(hardware_id()));
    println!("mqtt: device id: {}", device_id);
    loop {
        if let Err(_e) = run_mqtt_client(stack, module, &mqtt_config, device_id).await {
            println!("mqtt: connection lost, reconnecting in 2s...");
            embassy_time::Timer::after(Duration::from_secs(2)).await;
        }
    }
}

fn format_device_id(hardware_id: u32) -> String<17> {
    use core::fmt::Write;
    let mut device_id = String::<17>::new();
    let _ = write!(device_id, "myrtio-light-{:04X}", hardware_id);
    device_id
}

async fn run_mqtt_client(
    stack: Stack<'static>,
    module: &mut dyn MqttModule,
    mqtt_config: &MqttConfig,
    device_id: &'static str,
) -> Result<(), ()> {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(60)));

    let broker_addr = resolve_host(stack, mqtt_config.host.as_str()).await?;

    println!(
        "mqtt: connecting to broker {:?}:{}...",
        broker_addr, mqtt_config.port
    );

    let connection_result = socket.connect((broker_addr, mqtt_config.port)).await;
    if let Err(e) = connection_result {
        socket.abort();
        println!("mqtt: TCP connect failed: {:?}", e);
        return Err(());
    }
    println!("mqtt: TCP socket connected");

    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    let options = MqttOptions::new(device_id).with_keep_alive(Duration::from_secs(15));
    let mqtt: MqttClient<_, MQTT_MAX_TOPICS, MQTT_BUF_SIZE> = MqttClient::new(transport, options);

    // Create the runtime with the provided module
    let mut runtime: MqttRuntime<
        '_,
        _,
        &mut dyn MqttModule,
        MQTT_MAX_TOPICS,
        MQTT_BUF_SIZE,
        MQTT_OUTBOX_DEPTH,
    > = MqttRuntime::new(mqtt, module, PUBLISH_CHANNEL.receiver());

    runtime.run().await.map_err(|e| {
        println!("mqtt: runtime error: {:?}", e);
    })
}
