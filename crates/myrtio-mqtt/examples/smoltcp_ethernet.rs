//! Example: MQTT client with embassy-net TCP transport
//!
//! This example demonstrates how to:
//! - Connect to an MQTT broker
//! - Subscribe to a topic
//! - React to incoming messages
//! - Publish responses
//!
//! Note: This is a conceptual example. To run it, you need to integrate
//! it with your specific hardware setup and embassy-net stack initialization.

#![no_std]
#![no_main]

use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;
use myrtio_mqtt::client::{MqttClient, MqttEvent, MqttOptions};
use myrtio_mqtt::packet::QoS;
use myrtio_mqtt::transport::TcpTransport;

/// Command parsed from incoming message (owns its data, no borrowing)
enum Command {
    Status,
    Restart,
    Unknown,
}

/// Example MQTT task that subscribes to a topic and handles incoming messages.
async fn mqtt_task(socket: TcpSocket<'_>) {
    // Create transport with 30 second read timeout
    let transport = TcpTransport::new(socket, Duration::from_secs(30));

    // Configure MQTT client
    let options = MqttOptions::new("myrt-device-01", "192.168.1.100", 1883)
        .with_keep_alive(Duration::from_secs(60));

    // Create client with buffer sizes:
    // - MAX_TOPICS: 8 (maximum concurrent subscriptions)
    // - BUF_SIZE: 512 bytes for TX/RX buffers
    let mut client: MqttClient<_, 8, 512> = MqttClient::new(transport, options);

    // Connect to broker
    if let Err(_e) = client.connect().await {
        return;
    }

    // Subscribe to command topic
    if let Err(_e) = client.subscribe("device/commands", QoS::AtLeastOnce).await {
        return;
    }

    // Main event loop
    loop {
        // Extract command from event (releases borrow before using client again)
        let command = match client.poll().await {
            Ok(Some(MqttEvent::Publish(msg))) if msg.topic == "device/commands" => {
                match msg.payload {
                    b"status" => Command::Status,
                    b"restart" => Command::Restart,
                    _ => Command::Unknown,
                }
            }
            Ok(Some(_)) => continue,
            Ok(None) => continue,
            Err(_) => break,
        };

        // Now we can use client - the event borrow is released
        match command {
            Command::Status => {
                let _ = client.publish("device/status", b"online", QoS::AtMostOnce).await;
            }
            Command::Restart => {
                let _ = client.publish("device/status", b"restarting", QoS::AtMostOnce).await;
            }
            Command::Unknown => {
                let _ = client.publish("device/status", b"unknown command", QoS::AtMostOnce).await;
            }
        }
    }
}

// Placeholder main - in real firmware this would be embassy_executor::main
fn main() {
    // This example requires embassy runtime and hardware setup.
    // See embassy examples for full initialization:
    // https://github.com/embassy-rs/embassy/tree/main/examples
}
