/// Color mode
#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    Rgb,
    Temperature,
}

impl ColorMode {
    pub(crate) const fn as_u8(self) -> u8 {
        match self {
            ColorMode::Rgb => 0,
            ColorMode::Temperature => 1,
        }
    }

    pub(crate) const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ColorMode::Rgb),
            1 => Some(ColorMode::Temperature),
            _ => None,
        }
    }
}

/// Represents the light state.
#[derive(Debug, Clone)]
pub struct LightState {
    pub power: bool,
    pub brightness: u8,
    pub color: (u8, u8, u8),
    pub color_temp: u16,
    pub mode_id: u8,
    pub color_mode: ColorMode,
}

impl LightState {
    /// Create a new light state
    pub const fn new() -> Self {
        Self {
            power: true,
            brightness: 255,
            color: (255, 255, 255),
            color_temp: 4000,
            mode_id: 1, // rainbow
            color_mode: ColorMode::Rgb,
        }
    }
}

impl Default for LightState {
    /// Create a new light state with default values
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the boot sector slot.
#[derive(Debug, Clone, Copy)]
pub enum BootSlot {
    System,
    Factory,
}

impl BootSlot {
    pub(crate) const fn as_u8(self) -> u8 {
        match self {
            BootSlot::System => 0,
            BootSlot::Factory => 1,
        }
    }
}