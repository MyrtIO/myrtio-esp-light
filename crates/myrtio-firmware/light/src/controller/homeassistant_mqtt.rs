//! MQTT controller for light device
//!
//! Integrates with Home Assistant via MQTT discovery protocol.

use core::cell::RefCell;
use core::sync::atomic::{AtomicU8, Ordering};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::Duration;
use esp_println::println;
use myrtio_homeassistant::{ColorMode, Device, HomeAssistantClient, LightCommand, LightState};
use myrtio_light_composer::effect::{RainbowEffect, StaticColorEffect};
use myrtio_light_composer::{Command, CommandSender, EffectId, EffectSlot};
use myrtio_mqtt::{
    client::{MqttClient, MqttOptions},
    transport::TcpTransport,
};
use myrtio_netutils::resolve_host;

use crate::config::{self, LIGHT_LED_COUNT, MQTT_HOST, MQTT_PORT};
use crate::state::LIGHT_STATE;

/// Static device definition for Home Assistant
static DEVICE: Device<'static> = Device::builder(config::DEVICE_ID)
    .name(config::DEVICE_NAME)
    .manufacturer(config::DEVICE_MANUFACTURER)
    .model(config::DEVICE_MODEL)
    .build();

/// Effect names
const EFFECT_STATIC: &str = "static";
const EFFECT_RAINBOW: &str = "rainbow";

/// Duration (ms) for crossfading between colors
const COLOR_TRANSITION_MS: u64 = 300;

/// Target brightness to restore when turning on (not current display brightness)
static TARGET_BRIGHTNESS: AtomicU8 = AtomicU8::new(255);

/// Command sender (protected by mutex)
static COMMAND_SENDER: Mutex<CriticalSectionRawMutex, RefCell<Option<CommandSender<LIGHT_LED_COUNT>>>> =
    Mutex::new(RefCell::new(None));

/// Get effect name from EffectId
fn get_effect_name(id: EffectId) -> &'static str {
    match id {
        EffectId::Rainbow => EFFECT_RAINBOW,
        _ => EFFECT_STATIC,
    }
}

/// Get current light state for Home Assistant (reads from SharedState)
fn get_light_state() -> LightState<'static> {
    if LIGHT_STATE.is_on() {
        let (r, g, b) = LIGHT_STATE.rgb();
        LightState::on()
            .brightness(LIGHT_STATE.brightness())
            .rgb(r, g, b)
            .effect(get_effect_name(LIGHT_STATE.effect()))
    } else {
        LightState::off()
    }
}

/// Handle light command from Home Assistant
fn handle_light_command(cmd: LightCommand<'_>) {
    COMMAND_SENDER.lock(|cell| {
        let borrowed = cell.borrow();
        let Some(sender) = borrowed.as_ref() else {
            return;
        };
        let current_effect = LIGHT_STATE.effect();

        // Handle effect change
        if let Some(effect) = cmd.effect {
            match effect {
                EFFECT_RAINBOW => {
                    let _ = sender.try_send(Command::SwitchEffect(EffectSlot::Rainbow(
                        RainbowEffect::default(),
                    )));
                }
                _ => {
                    // Switch to static with current color from SharedState
                    let (r, g, b) = LIGHT_STATE.rgb();
                    let _ = sender.try_send(Command::SwitchEffect(EffectSlot::Static(
                        StaticColorEffect::from_rgb(r, g, b),
                    )));
                }
            }
        }

        // Handle color change
        if let Some(color) = cmd.color {
            if current_effect == EffectId::Static {
                let _ = sender.try_send(Command::SetColor {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    duration: Duration::from_millis(COLOR_TRANSITION_MS),
                });
            } else {
                let _ = sender.try_send(Command::SwitchEffect(EffectSlot::Static(
                    StaticColorEffect::from_rgb(color.r, color.g, color.b),
                )));
            }
        }

        // Handle brightness change (save target for restore on turn-on)
        if let Some(brightness) = cmd.brightness {
            TARGET_BRIGHTNESS.store(brightness, Ordering::Relaxed);
        }

        if cmd.is_off() {
            let _ = sender.try_send(Command::SetBrightness {
                brightness: 0,
                duration: Duration::from_millis(300),
            });
        } else if cmd.is_on() {
            let brightness = TARGET_BRIGHTNESS.load(Ordering::Relaxed);
            let _ = sender.try_send(Command::SetBrightness {
                brightness,
                duration: Duration::from_millis(300),
            });
        } else if cmd.brightness.is_some() {
            // Brightness-only change
            let brightness = TARGET_BRIGHTNESS.load(Ordering::Relaxed);
            let _ = sender.try_send(Command::SetBrightness {
                brightness,
                duration: Duration::from_millis(300),
            });
        }
    });
}

/// MQTT task that integrates with Home Assistant
#[embassy_executor::task]
pub async fn mqtt_controller_task(stack: Stack<'static>, sender: CommandSender<LIGHT_LED_COUNT>) {
    COMMAND_SENDER.lock(|cell| {
        cell.borrow_mut().replace(sender);
    });

    embassy_time::Timer::after(Duration::from_secs(1)).await;

    loop {
        if let Err(_e) = run_mqtt_client(stack).await {
            println!("MQTT connection lost, reconnecting in 5s...");
            embassy_time::Timer::after(Duration::from_secs(5)).await;
        }
    }
}

async fn run_mqtt_client(stack: Stack<'static>) -> Result<(), ()> {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(60)));

    let broker_addr = resolve_host(stack, MQTT_HOST).await?;
    println!("Resolved {} -> {:?}", MQTT_HOST, broker_addr);

    println!("Connecting to MQTT broker {}:{}...", MQTT_HOST, MQTT_PORT);
    socket
        .connect((broker_addr, MQTT_PORT))
        .await
        .map_err(|e| {
            println!("TCP connect failed: {:?}", e);
        })?;
    println!("TCP connected");

    let transport = TcpTransport::new(socket, Duration::from_secs(30));
    let options = MqttOptions::new(config::DEVICE_ID)
        .with_keep_alive(Duration::from_secs(30));
    let mqtt: MqttClient<_, 8, 512> = MqttClient::new(transport, options);

    // Create HA client and add entities
    let mut ha = HomeAssistantClient::<_, 8, 512, 4, 4>::new(&DEVICE, mqtt);

    ha.add_light(
        DEVICE
            .light()
            .name("Main Light")
            .icon("mdi:lightbulb")
            .brightness(true)
            .color_modes(&[ColorMode::Rgb])
            .provide_state(get_light_state)
            .on_command(handle_light_command)
            .effects(&[EFFECT_STATIC, EFFECT_RAINBOW])
            .build(),
    )
    .map_err(|e| {
        println!("Failed to add light: {:?}", e);
    })?;

    ha.run(Duration::from_secs(30)).await.map_err(|e| {
        println!("HA client error: {:?}", e);
    })
}
