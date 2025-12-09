use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::peripherals::RMT;

use myrtio_light_composer::color::rgb_from_u32;
use myrtio_light_composer::{
    CommandChannel, CommandSender, EffectProcessorConfig, LightEngine, LightEngineConfig, ModeId,
    Rgb, TransitionConfig,
};

use crate::infrastructure::config;
use crate::infrastructure::drivers::EspLedDriver;
use crate::infrastructure::types::LightDriver;

static LIGHT_COMMAND_CHANNEL: CommandChannel = Channel::new();

const LIGHT_COLOR_CORRECTION: Rgb = rgb_from_u32(config::LIGHT_COLOR_CORRECTION);
const LIGHT_CONFIG: LightEngineConfig = LightEngineConfig {
    mode: ModeId::Rainbow,
    brightness: 0,
    color: LIGHT_COLOR_CORRECTION,
    effects: EffectProcessorConfig {
        brightness_scale: Some(config::LIGHT_MAX_BRIGHTNESS_SCALE),
        color_correction: Some(LIGHT_COLOR_CORRECTION),
    },
    transition_config: TransitionConfig {
        fade_out_duration: Duration::from_millis(400),
        fade_in_duration: Duration::from_millis(300),
        color_change_duration: Duration::from_millis(200),
        brightness_change_duration: Duration::from_millis(200),
    },
};

/// Task for running the light composer
/// It receives commands from the command channel and updates the light state accordingly.
#[embassy_executor::task]
pub(crate) async fn light_composer_task(driver: LightDriver) {
    let receiver = LIGHT_COMMAND_CHANNEL.receiver();
    let mut engine: LightEngine<LightDriver, { config::LIGHT_LED_COUNT }> =
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
