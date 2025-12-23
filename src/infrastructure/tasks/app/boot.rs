use embassy_time::{Duration, Timer};

use crate::{
    controllers::BootController,
    domain::ports::OnBootHandler,
    infrastructure::repositories::{AppPersistentStorage, BootManager},
};

/// MQTT runtime task that accepts any module implementing `MqttModule`.
#[embassy_executor::task]
pub async fn boot_task(mut boot_controller: BootController<AppPersistentStorage, BootManager>) {
    Timer::after(Duration::from_secs(3)).await;
    boot_controller.on_magic_timeout();
}
