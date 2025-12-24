use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use esp_hal::{gpio::interconnect::PeripheralOutput, peripherals::RMT};
use myrtio_light_composer::{
    EffectProcessorConfig,
    IntentChannel,
    IntentReceiver,
    LightEngine,
    LightEngineConfig,
    LightIntent,
    ModeId,
    Rgb,
    bounds::RenderingBounds,
    color,
    effect::BrightnessEffectConfig,
    engine::LightStateIntent,
    ws2812_lut,
};

use super::state::AtomicLightState;
use crate::{
    config::{DEFAULT_TRANSITION_TIMINGS, LED_COUNT_MAX, LightConfig},
    domain::{
        dto::LightChangeIntent,
        entity::{ColorMode, LightState},
        ports::{
            LightConfigChanger,
            LightError,
            LightStateChanger,
            LightStateHandler,
            LightStateReader,
        },
    },
    infrastructure::{drivers::EspLedDriver, types::LightDriver},
};

const LIGHT_INTENT_CHANNEL_SIZE: usize = 10;

/// Channel for sending light intents to the light engine
static LIGHT_INTENT_CHANNEL: IntentChannel<LIGHT_INTENT_CHANNEL_SIZE> =
    Channel::new();

/// Global thread-safe lock-free light state
static LIGHT_STATE: AtomicLightState =
    AtomicLightState::from_state(&LightState::new());

#[derive(Debug, Default, Clone)]
pub struct LightStateService;

impl LightStateReader for LightStateService {
    fn get_light_state(&self) -> LightState {
        LIGHT_STATE.get()
    }
}

impl LightStateChanger for LightStateService {
    fn apply_light_intent(
        &self,
        intent: LightChangeIntent,
    ) -> Result<(), LightError> {
        let mut state = LIGHT_STATE.get();
        if let Some(mode_id_raw) = intent.mode_id {
            state.mode_id = mode_id_raw;
        }
        if let Some(brightness) = intent.brightness {
            state.brightness = brightness;
        }
        if let Some((r, g, b)) = intent.color {
            state.color = (r, g, b);
            state.color_mode = ColorMode::Rgb;
        } else if let Some(color_temp) = intent.color_temp {
            state.color_temp = color_temp;
            state.color_mode = ColorMode::Temperature;
        }

        if let Some(power) = intent.power {
            state.power = power;
        }

        let composer_intent = LightIntent::StateChange(LightStateIntent {
            power: intent.power,
            brightness: intent.brightness,
            color: intent.color.map(|(r, g, b)| Rgb { r, g, b }),
            color_temperature: intent.color_temp,
            mode_id: intent.mode_id.and_then(ModeId::from_raw),
        });
        send_intent_sync(composer_intent)?;
        LIGHT_STATE.set(&state);

        Ok(())
    }
}

impl LightConfigChanger for LightStateService {
    fn set_config(&mut self, config: LightConfig) -> Result<(), LightError> {
        let correction = color::rgb_from_u32(config.color_correction);
        let bounds = RenderingBounds {
            start: config.skip_leds,
            end: LED_COUNT_MAX as u8,
        };

        send_intent_sync(LightIntent::ColorCorrectionChange(correction))?;
        send_intent_sync(LightIntent::BoundsChange(bounds))?;
        Ok(())
    }
}

impl LightStateHandler for LightStateService {}

pub fn init_light_service<O>(
    spawner: Spawner,
    rmt: RMT<'static>,
    pin: O,
) -> LightStateService
where
    O: PeripheralOutput<'static>,
{
    let driver = EspLedDriver::new(rmt, pin);
    let config = LightEngineConfig {
        mode: ModeId::Static,
        brightness: 0,
        color: Rgb::new(255, 255, 255),
        bounds: RenderingBounds {
            start: 0,
            end: LED_COUNT_MAX as u8,
        },
        effects: EffectProcessorConfig {
            brightness: BrightnessEffectConfig {
                min_brightness: 0,
                scale: 255,
                adjust: Some(ws2812_lut),
            },
            color_correction: Some(color::rgb_from_u32(0xFF_FFFF)),
        },
        timings: DEFAULT_TRANSITION_TIMINGS,
    };
    let intents = LIGHT_INTENT_CHANNEL.receiver();

    spawner
        .spawn(light_engine_task(driver, intents, config))
        .expect("Failed to spawn light service task");

    LightStateService
}

/// Task for running the light engine
/// It receives intents from the intent channel and updates the light state
/// accordingly.
#[embassy_executor::task]
async fn light_engine_task(
    driver: LightDriver,
    intents: IntentReceiver<LIGHT_INTENT_CHANNEL_SIZE>,
    config: LightEngineConfig,
) {
    let mut engine: LightEngine<
        LightDriver,
        LED_COUNT_MAX,
        LIGHT_INTENT_CHANNEL_SIZE,
    > = LightEngine::new(driver, intents, &config);

    loop {
        engine.tick().await;
    }
}

fn send_intent_sync(intent: LightIntent) -> Result<(), LightError> {
    LIGHT_INTENT_CHANNEL
        .try_send(intent)
        .map_err(|_| LightError::Busy)
}
