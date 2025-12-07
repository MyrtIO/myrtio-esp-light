use myrtio_homeassistant::{ColorMode, Device, LightEntity};
use myrtio_light_composer::effect::EffectName;

use crate::infrastructure::config;

/// Static device definition for Home Assistant
pub(crate) static DEVICE: Device<'static> = Device::builder()
    .id(config::DEVICE_ID)
    .name(config::DEVICE_NAME)
    .manufacturer(Some(config::DEVICE_MANUFACTURER))
    .model(Some(config::DEVICE_MODEL))
    .build();

/// Static light entity definition
pub(crate) static LIGHT_ENTITY: LightEntity<'static> = LightEntity::builder()
    .id("led_strip")
    .name("LED Strip")
    .device(&DEVICE)
    .icon(Some("mdi:led-strip"))
    .brightness(true)
    .color_modes(&[ColorMode::Rgb])
    .effects(Some(&[
        EffectName::Static.as_str(),
        EffectName::Rainbow.as_str(),
    ]))
    .optimistic(false)
    .build();
