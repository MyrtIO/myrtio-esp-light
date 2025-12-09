use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::peripherals::RMT;

use myrtio_light_composer::color::rgb_from_u32;
use myrtio_light_composer::{
    CommandChannel, CommandSender, EffectProcessorConfig, LightEngine, LightEngineConfig, ModeId,
    Rgb, TransitionTimings,
};

use crate::infrastructure::config;
use crate::infrastructure::drivers::EspLedDriver;
use crate::infrastructure::types::LightDriver;

static LIGHT_COMMAND_CHANNEL: CommandChannel = Channel::new();

const LIGHT_COLOR_CORRECTION: Rgb = rgb_from_u32(config::LIGHT.color_correction);
const LIGHT_CONFIG: LightEngineConfig = LightEngineConfig {
    mode: ModeId::Rainbow,
    brightness: 0,
    color: LIGHT_COLOR_CORRECTION,
    effects: EffectProcessorConfig {
        brightness_min: Some(config::LIGHT.brightness_min),
        brightness_scale: Some(config::LIGHT.brightness_max),
        color_correction: Some(LIGHT_COLOR_CORRECTION),
    },
    timings: TransitionTimings {
        fade_out: Duration::from_millis(400),
        fade_in: Duration::from_millis(300),
        color_change: Duration::from_millis(200),
        brightness: Duration::from_millis(200),
    },
};

/// Task for running the light composer
/// It receives commands from the command channel and updates the light state accordingly.
#[embassy_executor::task]
pub(crate) async fn light_composer_task(driver: LightDriver) {
    let receiver = LIGHT_COMMAND_CHANNEL.receiver();
    let mut engine: LightEngine<LightDriver, { config::LIGHT.led_count }> =
        LightEngine::new(driver, receiver, &LIGHT_CONFIG);

    loop {
        engine.tick().await;
    }
}

pub(crate) fn init_light_composer<O>(rmt: RMT<'static>, pin: O) -> (LightDriver, CommandSender)
where
    O: PeripheralOutput<'static>,
{
    let driver = EspLedDriver::new(rmt, pin);

    (driver, LIGHT_COMMAND_CHANNEL.sender())
}
