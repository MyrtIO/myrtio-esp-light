//! OTA Invite Task
//!
//! This module implements an espota-style OTA update mechanism.
//! The device listens on a TCP port for update invitations, then downloads
//! and flashes new firmware from the specified HTTP server.
//!
//! The task is kept thin - it only handles networking and delegates
//! firmware update logic to the [`OtaController`].

use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;
use embedded_io_async::Write;
use esp_println::println;
use myrtio_core::net::resolve_host;

use crate::controllers::OtaController;
use crate::infrastructure::services::OtaInvite;

/// OTA listener port (standard espota port)
const OTA_PORT: u16 = 3232;

/// Buffer sizes for network operations
const LISTEN_RX_BUFFER_SIZE: usize = 512;
const LISTEN_TX_BUFFER_SIZE: usize = 128;
const DOWNLOAD_RX_BUFFER_SIZE: usize = 4096;
const DOWNLOAD_TX_BUFFER_SIZE: usize = 256;

/// OTA invite listener task
///
/// This task listens on a TCP port for OTA update invitations.
/// When an invite is received, it downloads the firmware from the
/// specified HTTP server and delegates flashing to the [`OtaController`].
///
/// After a successful update, the controller will trigger a reboot.
#[embassy_executor::task]
pub(crate) async fn ota_invite_task(stack: Stack<'static>, controller: &'static mut OtaController) {
    println!("ota: starting invite listener on port {}", OTA_PORT);

    loop {
        if let Err(e) = handle_ota_connection(stack, controller).await {
            println!("ota: connection error: {:?}", e);
        }
        // Small delay before accepting next connection
        embassy_time::Timer::after(Duration::from_millis(100)).await;
    }
}

/// OTA network error types
#[derive(Debug)]
enum OtaNetError {
    Accept,
    Read,
    Write,
    Parse,
    Resolve,
    Connect,
    Http,
    Controller,
}

/// Handle a single OTA connection
async fn handle_ota_connection(
    stack: Stack<'static>,
    controller: &mut OtaController,
) -> Result<(), OtaNetError> {
    let mut rx_buffer = [0u8; LISTEN_RX_BUFFER_SIZE];
    let mut tx_buffer = [0u8; LISTEN_TX_BUFFER_SIZE];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(30)));

    // Accept incoming connection
    socket
        .accept(OTA_PORT)
        .await
        .map_err(|_| OtaNetError::Accept)?;

    println!("ota: connection accepted");

    // Read invite data
    let mut invite_buffer = [0u8; 256];
    let mut total_read = 0;

    // Read until we have a complete invite or timeout
    loop {
        match socket.read(&mut invite_buffer[total_read..]).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                total_read += n;
                // Check for end of invite (double newline)
                if invite_buffer[..total_read].windows(2).any(|w| w == b"\n\n")
                    || invite_buffer[..total_read]
                        .windows(4)
                        .any(|w| w == b"\r\n\r\n")
                {
                    break;
                }
                if total_read >= invite_buffer.len() {
                    break;
                }
            }
            Err(_) => return Err(OtaNetError::Read),
        }
    }

    // Parse invite
    let invite = OtaInvite::parse(&invite_buffer[..total_read]).ok_or(OtaNetError::Parse)?;

    println!(
        "ota: received invite - host={}, port={}, path={}, size={}",
        invite.host.as_str(),
        invite.port,
        invite.path.as_str(),
        invite.size
    );

    // Send ACK
    socket
        .write_all(b"OK\n")
        .await
        .map_err(|_| OtaNetError::Write)?;

    // Close the invite socket
    socket.close();
    drop(socket);

    // Perform the update
    perform_ota_update(stack, &invite, controller).await
}

/// Download firmware and delegate flashing to the controller
async fn perform_ota_update(
    stack: Stack<'static>,
    invite: &OtaInvite,
    controller: &mut OtaController,
) -> Result<(), OtaNetError> {
    let mut rx_buffer = [0u8; DOWNLOAD_RX_BUFFER_SIZE];
    let mut tx_buffer = [0u8; DOWNLOAD_TX_BUFFER_SIZE];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(60)));

    // Resolve host
    let addr = resolve_host(stack, invite.host.as_str())
        .await
        .map_err(|()| OtaNetError::Resolve)?;

    println!("ota: connecting to {:?}:{}", addr, invite.port);

    // Connect to HTTP server
    socket
        .connect((addr, invite.port))
        .await
        .map_err(|_| OtaNetError::Connect)?;

    println!("ota: connected, sending HTTP request");

    // Send HTTP GET request
    let mut request = heapless::Vec::<u8, 256>::new();
    let _ = request.extend_from_slice(b"GET ");
    let _ = request.extend_from_slice(invite.path.as_bytes());
    let _ = request.extend_from_slice(b" HTTP/1.0\r\nHost: ");
    let _ = request.extend_from_slice(invite.host.as_bytes());
    let _ = request.extend_from_slice(b"\r\nConnection: close\r\n\r\n");

    socket
        .write_all(&request)
        .await
        .map_err(|_| OtaNetError::Write)?;

    // Read HTTP response headers
    let mut header_buffer = [0u8; 512];
    let mut header_len = 0;

    let body_start = loop {
        match socket.read(&mut header_buffer[header_len..]).await {
            Ok(0) => return Err(OtaNetError::Http),
            Ok(n) => {
                header_len += n;

                // Look for end of headers ("\r\n\r\n")
                if let Some(pos) = header_buffer[..header_len]
                    .windows(4)
                    .position(|w| w == b"\r\n\r\n")
                {
                    break pos + 4;
                }
                if header_len >= header_buffer.len() {
                    return Err(OtaNetError::Http);
                }
            }
            Err(_) => return Err(OtaNetError::Read),
        }
    };

    // Check for 200 OK (only parse the header part, exclude body bytes)
    let header_str = core::str::from_utf8(&header_buffer[..body_start]).unwrap_or("");
    if !header_str.starts_with("HTTP/1.") || !header_str.contains(" 200 ") {
        println!(
            "ota: HTTP error - {}",
            &header_str[..header_str.len().min(50)]
        );
        return Err(OtaNetError::Http);
    }

    println!("ota: HTTP 200 OK, starting download");

    // Start OTA via controller
    controller
        .start_ota(invite)
        .map_err(|()| OtaNetError::Controller)?;

    // Write any body data already in the header buffer
    if body_start < header_len {
        let initial_data = &header_buffer[body_start..header_len];
        controller
            .write_firmware_chunk(initial_data)
            .map_err(|()| OtaNetError::Controller)?;
    }

    // Read and write remaining data (progress is handled by controller)
    let mut read_buffer = [0u8; 4096];

    loop {
        match socket.read(&mut read_buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                controller
                    .write_firmware_chunk(&read_buffer[..n])
                    .map_err(|()| OtaNetError::Controller)?;
            }
            Err(_) => {
                // Abort the update on error
                controller.abort();
                return Err(OtaNetError::Read);
            }
        }
    }

    // Finalize the update (controller handles logging and reboot)
    controller
        .finish_ota()
        .map_err(|()| OtaNetError::Controller)?;

    // The controller will trigger a reboot, so we should not reach here
    Ok(())
}
