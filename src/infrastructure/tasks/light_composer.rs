use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::peripherals::RMT;

use myrtio_light_composer::color::rgb_from_u32;
use myrtio_light_composer::effect::BrightnessEffectConfig;
use myrtio_light_composer::{
    IntentChannel, IntentSender, EffectProcessorConfig, LightEngine, LightEngineConfig, ModeId,
    Rgb, TransitionTimings, ws2812_lut,
};

use crate::infrastructure::config;
use crate::infrastructure::drivers::EspLedDriver;
use crate::infrastructure::types::LightDriver;

static LIGHT_INTENT_CHANNEL: IntentChannel = Channel::new();

const LIGHT_COLOR_CORRECTION: Rgb = rgb_from_u32(config::LIGHT.color_correction);
const LIGHT_CONFIG: LightEngineConfig = LightEngineConfig {
    mode: ModeId::Rainbow,
    brightness: 0,
    color: LIGHT_COLOR_CORRECTION,
    effects: EffectProcessorConfig {
        brightness: BrightnessEffectConfig {
            min_brightness: config::LIGHT.brightness_min,
            scale: config::LIGHT.brightness_max,
            adjust: Some(ws2812_lut),
        },
        color_correction: Some(LIGHT_COLOR_CORRECTION),
    },
    timings: TransitionTimings {
        fade_out: Duration::from_millis(800),
        fade_in: Duration::from_millis(500),
        color_change: Duration::from_millis(200),
        brightness: Duration::from_millis(300),
    },
};

/// Task for running the light composer
/// It receives commands from the command channel and updates the light state accordingly.
#[embassy_executor::task]
pub async fn light_composer_task(driver: LightDriver) {
    let receiver = LIGHT_INTENT_CHANNEL.receiver();
    let mut engine: LightEngine<LightDriver, { config::LIGHT.led_count }> =
        LightEngine::new(driver, receiver, &LIGHT_CONFIG);

    loop {
        engine.tick().await;
    }
}

pub fn init_light_composer<O>(rmt: RMT<'static>, pin: O) -> (LightDriver, IntentSender)
where
    O: PeripheralOutput<'static>,
{
    let driver = EspLedDriver::new(rmt, pin);

    (driver, LIGHT_INTENT_CHANNEL.sender())
}
