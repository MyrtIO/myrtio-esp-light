use crate::{
    controllers::dependencies::LIGHT_USECASES,
    domain::{dto::LightChangeIntent, ports::OnBootHandler},
};

pub(crate) struct BootController;

impl BootController {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl OnBootHandler for BootController {
    fn on_boot(&self) {
        let stored_state = LIGHT_USECASES.lock(|cell| {
            let cell_ref = cell.borrow();
            cell_ref.as_ref().unwrap().get_persistent_light_state()
        });

        let state = stored_state.unwrap_or_default();
        let intent: LightChangeIntent = state.into();

        LIGHT_USECASES.lock(|cell| {
            let mut cell_ref = cell.borrow_mut();
            cell_ref.as_mut().unwrap().apply_intent(intent).unwrap();
        });
    }
}
