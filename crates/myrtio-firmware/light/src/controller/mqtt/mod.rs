//! MQTT controller for light device
//!
//! Integrates with Home Assistant via MQTT discovery protocol.

use core::cell::RefCell;
use core::sync::atomic::{AtomicU8, Ordering};
use embassy_futures::select::{Either, select};
use embassy_net::{IpAddress, Stack, dns::DnsQueryType, tcp::TcpSocket};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use embassy_time::{Duration, Ticker};
use esp_println::println;
use myrtio_light_composer::{Command, CommandSender};
use myrtio_mqtt::{
    client::{MqttClient, MqttOptions},
    transport::TcpTransport,
};
use myrtio_homeassistant::{
    Device, HomeAssistantClient, LightBuilder, LightCommand, LightState,
};

use crate::config::{MQTT_HOST, NUM_LEDS};

const MQTT_PORT: u16 = 1883;
const MQTT_NAMESPACE: &str = "myrtlight";

/// Static device definition for Home Assistant
static DEVICE: Device<'static> = Device::builder("myrt_light_rs")
    .name("Myrt Light RS")
    .manufacturer("Myrtio")
    .model("Light v1")
    .build();

/// Global brightness state (shared between callbacks and main task)
static CURRENT_BRIGHTNESS: AtomicU8 = AtomicU8::new(0);
static IS_ON: AtomicU8 = AtomicU8::new(0);

/// Command sender - set during initialization (protected by mutex)
static COMMAND_SENDER: Mutex<CriticalSectionRawMutex, RefCell<Option<CommandSender<NUM_LEDS>>>> =
    Mutex::new(RefCell::new(None));

/// Get current light state for Home Assistant
fn get_light_state() -> LightState<'static> {
    let brightness = CURRENT_BRIGHTNESS.load(Ordering::Relaxed);
    let is_on = IS_ON.load(Ordering::Relaxed) != 0;
    
    if is_on && brightness > 0 {
        LightState::on().brightness(brightness)
    } else {
        LightState::off()
    }
}

/// Handle light command from Home Assistant
fn handle_light_command(cmd: LightCommand<'_>) {
    COMMAND_SENDER.lock(|cell| {
        let borrowed = cell.borrow();
        let Some(sender) = borrowed.as_ref() else {
            println!("Command sender not initialized");
            return;
        };
        
        if cmd.is_off() {
            IS_ON.store(0, Ordering::Relaxed);
            CURRENT_BRIGHTNESS.store(0, Ordering::Relaxed);
            
            // Send stop command (non-blocking)
            let _ = sender.try_send(Command::SetBrightness {
                brightness: 0,
                duration: Duration::from_millis(300),
            });
            println!("Light OFF");
        } else if cmd.is_on() {
            let brightness = cmd.brightness.unwrap_or(255);
            IS_ON.store(1, Ordering::Relaxed);
            CURRENT_BRIGHTNESS.store(brightness, Ordering::Relaxed);
            
            let _ = sender.try_send(Command::SetBrightness {
                brightness,
                duration: Duration::from_millis(300),
            });
            println!("Light ON, brightness: {}", brightness);
        } else if let Some(brightness) = cmd.brightness {
            // Just brightness change without state change
            CURRENT_BRIGHTNESS.store(brightness, Ordering::Relaxed);
            
            let _ = sender.try_send(Command::SetBrightness {
                brightness,
                duration: Duration::from_millis(300),
            });
            println!("Brightness: {}", brightness);
        }
    });
}

/// Resolves a hostname to an IP address.
/// First tries to parse as an IP address, then falls back to DNS query.
async fn resolve_host(stack: Stack<'static>, host: &str) -> Result<IpAddress, ()> {
    // First try to parse as IP address
    if let Ok(ip) = host.parse::<embassy_net::Ipv4Address>() {
        return Ok(IpAddress::Ipv4(ip));
    }

    // Fallback to DNS query
    println!("Resolving hostname: {}", host);
    let addrs = stack.dns_query(host, DnsQueryType::A).await.map_err(|e| {
        println!("DNS query failed: {:?}", e);
    })?;

    addrs.first().copied().ok_or_else(|| {
        println!("No DNS records found for {}", host);
    })
}

/// MQTT task that integrates with Home Assistant
#[embassy_executor::task]
pub async fn mqtt_controller_task(stack: Stack<'static>, sender: CommandSender<NUM_LEDS>) {
    // Store the command sender for use in callbacks
    COMMAND_SENDER.lock(|cell| {
        cell.borrow_mut().replace(sender);
    });
    
    // Wait a bit for network to stabilize
    embassy_time::Timer::after(Duration::from_secs(1)).await;

    loop {
        if let Err(_e) = run_mqtt_client(stack).await {
            println!("MQTT connection lost, reconnecting in 5s...");
            embassy_time::Timer::after(Duration::from_secs(5)).await;
        }
    }
}

async fn run_mqtt_client(stack: Stack<'static>) -> Result<(), ()> {
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

    let mqtt: MqttClient<_, 8, 512> = MqttClient::new(transport, options);
    
    // Create Home Assistant client
    let mut ha = HomeAssistantClient::<_, 8, 512, 4, 4>::new(mqtt, MQTT_NAMESPACE);
    
    // Connect to MQTT broker
    ha.mqtt_mut().connect().await.map_err(|e| {
        println!("MQTT connect failed: {:?}", e);
    })?;
    println!("MQTT connected");
    
    // Define light entity
    let light = LightBuilder::new("main", &DEVICE)
        .name("Main Light")
        .icon("mdi:lightbulb")
        .brightness(true)
        .build();
    
    // Register light entity with callbacks
    ha.register_light(light, get_light_state, handle_light_command)
        .map_err(|e| {
            println!("Failed to register light: {:?}", e);
        })?;
    
    // Announce to Home Assistant
    ha.announce_all().await.map_err(|e| {
        println!("Failed to announce: {:?}", e);
    })?;
    println!("Announced to Home Assistant");
    
    // Publish initial state
    ha.publish_states().await.map_err(|e| {
        println!("Failed to publish initial state: {:?}", e);
    })?;

    let mut state_ticker = Ticker::every(Duration::from_secs(30));

    // Main event loop
    loop {
        match select(ha.poll(), state_ticker.next()).await {
            Either::First(poll_result) => {
                match poll_result {
                    Ok(true) => {
                        // Command was handled, publish updated state
                        let _ = ha.publish_states().await;
                    }
                    Ok(false) => {
                        // No command, continue
                    }
                    Err(e) => {
                        println!("HA poll error: {:?}", e);
                        return Err(());
                    }
                }
            }
            Either::Second(_) => {
                // Periodic state publish
                let _ = ha.publish_states().await;
            }
        }
    }
}
