//! Generic HTTP Server Adapter
//!
//! Provides a generic `run_http_server` function that:
//! 1. Allocates RX/TX buffers on the stack.
//! 2. Initializes the `HttpServer` with the provided handler.
//! 3. Runs the `listen_and_serve` loop.
//!
//! This function is not an Embassy task itself; it is meant to be called from
//! a task (e.g. `factory_http_server_task`) or directly from `main` if appropriate.

use embassy_net::Stack;
use esp_println::println;

use crate::core::net::http::{HttpHandler, HttpServer};

const HTTP_PORT: u16 = 80;
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;

/// Run the HTTP server with the given handler.
///
/// This function allocates 8KB of buffers on the stack (4KB RX + 4KB TX).
/// Ensure the calling task has sufficient stack size!
pub async fn run_http_server<H: HttpHandler>(stack: Stack<'static>, handler: &H) {
    let server = HttpServer::<H, TX_BUFFER_SIZE, RX_BUFFER_SIZE>::new(handler);
    let mut rx_buffer = [0u8; RX_BUFFER_SIZE];
    let mut tx_buffer = [0u8; TX_BUFFER_SIZE];

    if let Err(e) = server
        .listen_and_serve(stack, HTTP_PORT, &mut rx_buffer, &mut tx_buffer)
        .await
    {
        println!("http_server: connection error: {:?}", e);
    }
}
