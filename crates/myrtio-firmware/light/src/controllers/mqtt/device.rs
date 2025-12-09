use myrtio_homeassistant::{ColorMode, Device, LightEntity};
use myrtio_light_composer::ModeId;

use crate::infrastructure::config;

/// Static device definition for Home Assistant
pub(crate) static DEVICE: Device<'static> = Device::builder()
    .id(config::DEVICE.id)
    .name(config::DEVICE.name)
    .manufacturer(Some(config::DEVICE.manufacturer))
    .model(Some(config::DEVICE.model))
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
        ModeId::Static.as_str(),
        ModeId::Rainbow.as_str(),
    ]))
    .optimistic(false)
    .build();
