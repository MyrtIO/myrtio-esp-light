/// Represents the light state.
#[derive(Clone, Debug)]
pub(crate) struct LightState {
    pub power: bool,
    pub brightness: u8,
    pub color: (u8, u8, u8),
    pub effect_id: u8,
}

impl Default for LightState {
    fn default() -> Self {
        Self {
            power: true,
            brightness: 255,
            color: (255, 255, 255),
            effect_id: 1 // rainbow
        }
    }
}
