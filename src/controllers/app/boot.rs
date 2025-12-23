use super::LIGHT_USECASES;
use crate::domain::dto::LightChangeIntent;
use crate::domain::entity::LightState;
use crate::domain::ports::OnBootHandler;

#[derive(Default)]
pub struct BootController;

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
