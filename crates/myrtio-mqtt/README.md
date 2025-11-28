# Async Embedded MQTT Client

An `async`, `no_std`-compatible MQTT client library in Rust, designed for embedded systems using the [Embassy](https://embassy.dev/) async ecosystem and `embedded-hal` 1.0.0 traits.

## Core Features

- **Asynchronous:** Built on `async/await` and designed for the Embassy ecosystem.
- **`no_std` by default:** Suitable for bare-metal and resource-constrained devices.
- **Hardware Agnostic:** Uses `embedded-hal-async` traits to support various communication transports (TCP, UART, SPI, etc.).
- **Memory Efficient:** Leverages `heapless` to avoid dynamic memory allocation.
- **MQTT v3.1.1 and v5 Support:** Protocol version can be selected via feature flags.
- **QoS 0 & 1:** Support for "at most once" and "at least once" message delivery.
- **Embassy-net Integration:** Built-in `TcpTransport` for `embassy-net` TCP sockets.

## Getting Started

### Basic Usage

```rust,no_run
use myrtio_mqtt::{MqttClient, MqttEvent, MqttOptions, QoS, TcpTransport};
use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;

async fn run_mqtt(socket: TcpSocket<'_>) {
    // Create transport with timeout
    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    
    // Configure client
    let options = MqttOptions::new("my-device", "192.168.1.100", 1883)
        .with_keep_alive(Duration::from_secs(60));
    
    let mut client: MqttClient<_, 8, 512> = MqttClient::new(transport, options);

    // Connect to broker
    client.connect().await.unwrap();

    // Publish a message
    client.publish("sensors/temp", b"25.3", QoS::AtLeastOnce).await.unwrap();
}
```

### Subscribe and Handle Messages

```rust,no_run
use myrtio_mqtt::{MqttClient, MqttEvent, MqttOptions, QoS, TcpTransport};
use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;

async fn mqtt_subscriber(socket: TcpSocket<'_>) {
    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    let options = MqttOptions::new("my-device", "192.168.1.100", 1883);
    let mut client: MqttClient<_, 8, 512> = MqttClient::new(transport, options);

    // Connect and subscribe
    client.connect().await.unwrap();
    client.subscribe("device/commands", QoS::AtLeastOnce).await.unwrap();

    // Event loop - handle incoming messages
    loop {
        match client.poll().await {
            Ok(Some(MqttEvent::Publish(msg))) => {
                // Process incoming message
                let topic = msg.topic;
                let payload = msg.payload;
                
                if topic == "device/commands" {
                    match payload {
                        b"status" => {
                            client.publish("device/status", b"online", QoS::AtMostOnce).await.ok();
                        }
                        b"restart" => {
                            // Handle restart command
                        }
                        _ => {
                            // Echo unknown commands
                            client.publish("device/echo", payload, QoS::AtMostOnce).await.ok();
                        }
                    }
                }
            }
            Ok(None) => {
                // Timeout - continue polling
            }
            Err(_) => {
                // Connection lost
                break;
            }
        }
    }
}
```

## Project Structure

The library is organized into a few key modules:

- `src/client.rs`: Contains the main `MqttClient` and its async state machine.
- `src/packet.rs`: Handles the encoding and decoding of MQTT control packets.
- `src/transport.rs`: Defines the `MqttTransport` trait and `TcpTransport` implementation.
- `src/error.rs`: Provides unified error types for the client.

## Feature Flags

The following feature flags are available:

- `v5`: Enables MQTT v5 support.
- `defmt`: Enables logging via the `defmt` framework.

## Examples

See the `examples/` directory for complete examples:

- `smoltcp_ethernet.rs`: Full example with subscription and message handling using embassy-net.
