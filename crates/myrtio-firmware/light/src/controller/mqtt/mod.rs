//! MQTT controller for light device
//!
//! Subscribes to command topic and controls light brightness.

use embassy_net::{dns::DnsQueryType, tcp::TcpSocket, IpAddress, Stack};
use embassy_time::Duration;
use esp_println::println;
use myrtio_light_composer::{Command, CommandSender};
use myrtio_mqtt::{
    client::{MqttClient, MqttEvent, MqttOptions},
    transport::TcpTransport,
    QoS,
};

use crate::config::{MQTT_HOST, NUM_LEDS};

const MQTT_PORT: u16 = 1883;
const COMMAND_TOPIC: &str = "myrtlightrs/command";

/// Resolves a hostname to an IP address.
/// First tries to parse as an IP address, then falls back to DNS query.
async fn resolve_host(stack: Stack<'static>, host: &str) -> Result<IpAddress, ()> {
    // First try to parse as IP address
    if let Ok(ip) = host.parse::<embassy_net::Ipv4Address>() {
        return Ok(IpAddress::Ipv4(ip));
    }

    // Fallback to DNS query
    println!("Resolving hostname: {}", host);
    let addrs = stack
        .dns_query(host, DnsQueryType::A)
        .await
        .map_err(|e| {
            println!("DNS query failed: {:?}", e);
        })?;

    addrs.first().copied().ok_or_else(|| {
        println!("No DNS records found for {}", host);
    })
}

/// MQTT task that subscribes to command topic and controls light
#[embassy_executor::task]
pub async fn mqtt_controller_task(stack: Stack<'static>, sender: CommandSender<NUM_LEDS>) {
    // Wait a bit for network to stabilize
    embassy_time::Timer::after(Duration::from_secs(1)).await;

    loop {
        if let Err(_e) = run_mqtt_client(stack, sender.clone()).await {
            println!("MQTT connection lost, reconnecting in 5s...");
            embassy_time::Timer::after(Duration::from_secs(5)).await;
        }
    }
}

async fn run_mqtt_client(
    stack: Stack<'static>,
    sender: CommandSender<NUM_LEDS>,
) -> Result<(), ()> {
    // Create socket buffers
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(60)));

    // Resolve broker address (supports both IP and hostname)
    let broker_addr = resolve_host(stack, MQTT_HOST).await?;
    println!("Resolved {} -> {:?}", MQTT_HOST, broker_addr);

    // Connect TCP socket to broker
    println!("Connecting to MQTT broker {}:{}...", MQTT_HOST, MQTT_PORT);
    socket
        .connect((broker_addr, MQTT_PORT))
        .await
        .map_err(|e| {
            println!("TCP connect failed: {:?}", e);
        })?;

    println!("TCP connected");

    // Create MQTT transport and client
    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    let options = MqttOptions::new("myrt-light-rs", MQTT_HOST, MQTT_PORT)
        .with_keep_alive(Duration::from_secs(30));

    let mut client: MqttClient<_, 8, 512> = MqttClient::new(transport, options);

    // Connect to MQTT broker
    client.connect().await.map_err(|e| {
        println!("MQTT connect failed: {:?}", e);
    })?;
    println!("MQTT connected");

    // Subscribe to command topic
    client
        .subscribe(COMMAND_TOPIC, QoS::AtLeastOnce)
        .await
        .map_err(|e| {
            println!("Subscribe failed: {:?}", e);
        })?;
    println!("Subscribed to {}", COMMAND_TOPIC);

    // Main event loop
    loop {
        let command = match client.poll().await {
            Ok(Some(MqttEvent::Publish(msg))) if msg.topic == COMMAND_TOPIC => {
                match msg.payload {
                    b"on" => Some(50u8),
                    b"off" => Some(0u8),
                    _ => {
                        println!("Unknown command: {:?}", msg.payload);
                        None
                    }
                }
            }
            Ok(_) => continue,
            Err(e) => {
                println!("MQTT poll error: {:?}", e);
                return Err(());
            }
        };

        if let Some(brightness) = command {
            sender
                .send(Command::SetBrightness {
                    brightness,
                    duration: Duration::from_millis(300),
                })
                .await;
        }
    }
}

