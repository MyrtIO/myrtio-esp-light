use core::future::Future;

use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
#[cfg(feature = "log")]
use esp_println::println;

use super::{HttpResult, connection::HttpConnection};

pub trait HttpHandler {
    fn handle_request<'a>(
        &'a self,
        conn: HttpConnection<'a>,
    ) -> impl Future<Output = HttpResult> + 'a;
}

pub(crate) struct HttpServer<
    'a,
    T: HttpHandler + ?Sized,
    const TX_SIZE: usize,
    const RX_SIZE: usize,
> {
    handler: &'a T,
}

impl<'a, T: HttpHandler + ?Sized, const TX_SIZE: usize, const RX_SIZE: usize>
    HttpServer<'a, T, TX_SIZE, RX_SIZE>
{
    pub(crate) fn new(handler: &'a T) -> Self {
        Self { handler }
    }
}

impl<T: HttpHandler + ?Sized, const TX_SIZE: usize, const RX_SIZE: usize>
    HttpServer<'_, T, TX_SIZE, RX_SIZE>
{
    pub(crate) async fn listen_and_serve(
        &self,
        stack: Stack<'static>,
        port: u16,
        rx_buffer: &mut [u8],
        tx_buffer: &mut [u8],
    ) -> HttpResult {
        loop {
            let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
            socket.set_timeout(Some(Duration::from_secs(30)));

            if socket.accept(port).await.is_err() {
                continue;
            }

            let conn = match HttpConnection::from_socket(socket).await {
                Ok(connection) => connection,
                Err(_e) => {
                    #[cfg(feature = "log")]
                    println!("http_server: connection startup error: {:?}", _e);
                    continue;
                }
            };

            if let Err(_e) = self.handler.handle_request(conn).await {
                #[cfg(feature = "log")]
                println!("http_server: handler error: {:?}", _e);
            }
        }
    }
}
