use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::peripherals::RMT;

use myrtio_light_composer::bounds::RenderingBounds;
use myrtio_light_composer::color::rgb_from_u32;
use myrtio_light_composer::effect::BrightnessEffectConfig;
use myrtio_light_composer::{
    EffectProcessorConfig, IntentChannel, IntentSender, LightEngine, LightEngineConfig, ModeId,
    Rgb, TransitionTimings, ws2812_lut,
};

use crate::infrastructure::drivers::EspLedDriver;
use crate::infrastructure::types::LightDriver;

static LIGHT_INTENT_CHANNEL: IntentChannel = Channel::new();

pub struct LightTaskParams {
    pub min_brightness: u8,
    pub max_brightness: u8,
    pub led_count: u8,
    pub skip_leds: u8,
    pub color_correction: u32,
}

/// Task for running the light composer
/// It receives commands from the command channel and updates the light state accordingly.
#[embassy_executor::task]
pub async fn light_composer_task(driver: LightDriver, params: LightTaskParams) {
    let config = LightEngineConfig {
        mode: ModeId::Rainbow,
        brightness: 0,
        color: Rgb::new(255, 255, 255),
        bounds: RenderingBounds {
            start: params.skip_leds,
            end: params.skip_leds + params.led_count,
        },
        effects: EffectProcessorConfig {
            brightness: BrightnessEffectConfig {
                min_brightness: params.min_brightness,
                scale: params.max_brightness,
                adjust: Some(ws2812_lut),
            },
            color_correction: Some(rgb_from_u32(params.color_correction)),
        },
        timings: TransitionTimings {
            fade_out: Duration::from_millis(800),
            fade_in: Duration::from_millis(500),
            color_change: Duration::from_millis(200),
            brightness: Duration::from_millis(300),
        },
    };

    let receiver = LIGHT_INTENT_CHANNEL.receiver();
    let mut engine: LightEngine<LightDriver, 128> = LightEngine::new(driver, receiver, &config);

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
