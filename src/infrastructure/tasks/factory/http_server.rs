//! Factory HTTP Server Task
//!
//! Spawns the HTTP server with the `FactoryHttpController`.

use embassy_net::Stack;

use crate::controllers::factory::FactoryHttpController;
use crate::infrastructure::adapters::http_server::run_http_server;

#[embassy_executor::task]
pub async fn http_server_task(
    stack: Stack<'static>,
    handler: &'static FactoryHttpController,
) {
    run_http_server(stack, handler).await;
}
