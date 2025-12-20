use crate::{
    controllers::dependencies::LIGHT_USECASES,
    domain::{dto::LightChangeIntent, entity::LightState, ports::OnBootHandler},
};

pub(crate) struct BootController;

impl BootController {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl OnBootHandler for BootController {
    fn on_boot(&self, stored_state: Option<LightState>) {
        let state = stored_state.unwrap_or_default();
        let intent: LightChangeIntent = state.into();

        LIGHT_USECASES.lock(|cell| {
            let mut cell_ref = cell.borrow_mut();
            cell_ref.as_mut().unwrap().apply_intent(intent).unwrap();
        });
    }
}
