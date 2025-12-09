//! MQTT Runtime Task
//!
//! This module provides the infrastructure-level MQTT task that accepts any
//! `MqttModule` implementation via a trait object.

use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_println::println;
use myrtio_core::net::resolve_host;
use myrtio_mqtt::runtime::{MqttModule, MqttRuntime, PublishRequestChannel};
use myrtio_mqtt::{
    client::{MqttClient, MqttOptions},
    transport::TcpTransport,
};

use crate::infrastructure::config;

const MQTT_OUTBOX_DEPTH: usize = 4;
const MQTT_MAX_TOPICS: usize = 8;
const MQTT_BUF_SIZE: usize = 512;

/// Static channel for publish requests (used by the runtime)
static PUBLISH_CHANNEL: PublishRequestChannel<'static, MQTT_OUTBOX_DEPTH> = Channel::new();

/// MQTT runtime task that accepts any module implementing `MqttModule`.
///
/// This task handles:
/// - TCP connection management
/// - MQTT client lifecycle
/// - Reconnection on failure
///
/// The module is passed as a `&'static mut dyn MqttModule` trait object,
/// allowing the infrastructure to be completely decoupled from any specific
/// module implementation.
#[embassy_executor::task]
pub async fn mqtt_runtime_task(stack: Stack<'static>, module: &'static mut dyn MqttModule) {
    loop {
        if let Err(_e) = run_mqtt_client(stack, module).await {
            println!("MQTT connection lost, reconnecting in 2s...");
            embassy_time::Timer::after(Duration::from_secs(2)).await;
        }
    }
}

async fn run_mqtt_client(stack: Stack<'static>, module: &mut dyn MqttModule) -> Result<(), ()> {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(60)));

    let broker_addr = resolve_host(stack, config::MQTT.host).await?;

    println!(
        "Connecting to MQTT broker {:?}:{}...",
        broker_addr,
        config::MQTT.port
    );
    socket
        .connect((broker_addr, config::MQTT.port))
        .await
        .map_err(|e| {
            println!("TCP connect failed: {:?}", e);
        })?;
    println!("TCP connected");

    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    let options = MqttOptions::new(config::DEVICE.id).with_keep_alive(Duration::from_secs(15));
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
        println!("MQTT runtime error: {:?}", e);
    })
}
