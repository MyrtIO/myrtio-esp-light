/// Represents the light state.
#[derive(Clone, Debug)]
pub(crate) struct LightState {
    pub power: bool,
    pub brightness: u8,
    pub color: (u8, u8, u8),
    pub mode_id: u8,
}

impl LightState {
    /// Create a new light state
    pub(crate) const fn new() -> Self {
        Self {
            power: true,
            brightness: 255,
            color: (255, 255, 255),
            mode_id: 1, // rainbow
        }
    }
}

impl Default for LightState {
    /// Create a new light state with default values
    fn default() -> Self {
        Self::new()
    }
}
