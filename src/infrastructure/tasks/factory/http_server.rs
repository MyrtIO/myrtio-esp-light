//! Factory HTTP Server
//!
//! Provides a web interface for device provisioning and OTA updates.
//! Serves a configuration page and handles config saves and firmware uploads.

use crate::controllers::FactoryHttpController;
use crate::infrastructure::services::http::http_server::HttpServer;
use embassy_net::Stack;
use esp_println::println;

const HTTP_PORT: u16 = 80;
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;

#[embassy_executor::task]
pub async fn http_server_task(stack: Stack<'static>, handler: FactoryHttpController) {
    let server = HttpServer::<FactoryHttpController, RX_BUFFER_SIZE, TX_BUFFER_SIZE>::new(&handler);
    let mut rx_buffer = [0u8; RX_BUFFER_SIZE];
    let mut tx_buffer = [0u8; TX_BUFFER_SIZE];

    if let Err(e) = server
        .listen_and_serve(stack, HTTP_PORT, &mut rx_buffer, &mut tx_buffer)
        .await
    {
        println!("factory_http: connection error: {:?}", e);
    }
}
