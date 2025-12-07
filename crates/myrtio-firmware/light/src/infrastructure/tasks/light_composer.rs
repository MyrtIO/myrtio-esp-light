use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::peripherals::RMT;
use myrtio_light_composer::CommandChannel;
use myrtio_light_composer::effect::StaticColorEffect;
use myrtio_light_composer::{ColorCorrection, EffectSlot, LightEngine};

use crate::infrastructure::config;
use crate::infrastructure::drivers::EspLedDriver;
use crate::infrastructure::types::{LightCommandSender, LightDriver};

static LIGHT_COMMAND_CHANNEL: CommandChannel<{ config::LIGHT_LED_COUNT }> = Channel::new();

const LIGHT_COLOR_CORRECTION: ColorCorrection =
    ColorCorrection::from_rgb(config::LIGHT_COLOR_CORRECTION);

/// Task for running the light composer
/// It receives commands from the command channel and updates the light state accordingly.
#[embassy_executor::task]
pub(crate) async fn light_composer_task(driver: LightDriver) {
    let receiver = LIGHT_COMMAND_CHANNEL.receiver();
    let mut engine =
        LightEngine::new(driver, receiver).with_color_correction(LIGHT_COLOR_CORRECTION).with_brightness_scale(config::LIGHT_MAX_BRIGHTNESS_SCALE);

    engine.set_brightness(0, Duration::from_millis(0));
    let effect = EffectSlot::Static(StaticColorEffect::default());
    engine.set_effect(effect);

    loop {
        engine.tick().await;
    }
}

pub(crate) fn init_light_composer<O>(rmt: RMT<'static>, pin: O) -> (LightDriver, LightCommandSender)
where
    O: PeripheralOutput<'static>,
{
    let driver = EspLedDriver::<{ config::LIGHT_LED_COUNT }>::new(rmt, pin);

    (driver, LIGHT_COMMAND_CHANNEL.sender())
}
