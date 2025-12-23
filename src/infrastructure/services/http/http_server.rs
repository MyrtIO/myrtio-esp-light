use embassy_time::Duration;

use super::HttpResult;
use super::connection::HttpConnection;

use embassy_net::{Stack, tcp::TcpSocket};
use esp_println::println;

pub trait HttpHandler {
    async fn handle_request(&self, conn: HttpConnection<'_>) -> HttpResult;
}

pub struct HttpServer<'a, T: HttpHandler, const TX_SIZE: usize, const RX_SIZE: usize> {
    handler: &'a T,
}

impl<'a, T: HttpHandler, const TX_SIZE: usize, const RX_SIZE: usize>
    HttpServer<'a, T, TX_SIZE, RX_SIZE>
{
    pub fn new(handler: &'a T) -> Self {
        Self { handler }
    }
}

impl<'a, T: HttpHandler, const TX_SIZE: usize, const RX_SIZE: usize>
    HttpServer<'a, T, TX_SIZE, RX_SIZE>
{
    pub async fn listen_and_serve(&self, stack: Stack<'static>, port: u16, rx_buffer: &mut [u8], tx_buffer: &mut [u8]) -> HttpResult {
        loop {
            let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
            socket.set_timeout(Some(Duration::from_secs(30)));

            if socket.accept(port).await.is_err() {
                continue;
            }

            if let Err(e) = self.handle_connection(socket).await {
                println!("http_server: connection error: {:?}", e);
                // continue;
            }
        }
    }

    async fn handle_connection<'b>(&self, socket: TcpSocket<'b>) -> HttpResult {
        let connection = HttpConnection::from_socket(socket).await?;

        self.handler.handle_request(connection).await
    }
}
