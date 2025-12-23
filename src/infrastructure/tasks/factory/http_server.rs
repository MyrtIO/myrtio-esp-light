//! Factory HTTP Server
//!
//! Provides a web interface for device provisioning and OTA updates.
//! Serves a configuration page and handles config saves and firmware uploads.

use core::fmt::Write;
use core::str;

use embassy_net::tcp::TcpSocket;
use embassy_net::Stack;
use embassy_time::Duration;
use embedded_io_async::Write as AsyncWrite;
use embedded_storage::nor_flash::ReadNorFlash;
use esp_bootloader_esp_idf::ota::OtaImageState;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN;
use esp_println::println;
use esp_storage::FlashStorage;

use crate::config::{DeviceConfig, LightConfig, MqttConfig, WifiConfig, CONFIGURATION_PARTITION_OFFSET};
use crate::domain::ports::PersistenceHandler;
use crate::infrastructure::repositories::AppPersistentStorage;

const HTTP_PORT: u16 = 80;
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;
const CHUNK_SIZE: usize = 1024;
const HEADER_BUFFER_SIZE: usize = 2048;

/// Factory HTTP server task
///
/// Handles web requests for configuration and OTA updates.
#[embassy_executor::task]
pub async fn factory_http_server_task(stack: Stack<'static>, flash: *mut FlashStorage<'static>) {
    println!("factory_http: starting on port {}", HTTP_PORT);

    loop {
        let mut rx_buffer = [0u8; RX_BUFFER_SIZE];
        let mut tx_buffer = [0u8; TX_BUFFER_SIZE];

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(30)));

        if socket.accept(HTTP_PORT).await.is_err() {
            continue;
        }

        if let Err(e) = handle_connection(&mut socket, flash).await {
            println!("factory_http: connection error: {:?}", e);
        }

        // Flush and close gracefully
        let _ = socket.flush().await;
        socket.close();
    }
}

#[derive(Debug)]
enum HttpError {
    Read,
    Write,
    Parse,
    Flash,
}

async fn handle_connection<'a>(
    socket: &mut TcpSocket<'a>,
    flash: *mut FlashStorage<'static>,
) -> Result<(), HttpError> {
    let mut header_buf = [0u8; HEADER_BUFFER_SIZE];
    let mut header_len = 0;

    // Read headers
    let mut header_end = None;
    loop {
        let n = socket.read(&mut header_buf[header_len..]).await.map_err(|_| HttpError::Read)?;
        if n == 0 {
            return Err(HttpError::Read);
        }
        header_len += n;

        // Check for end of headers
        if let Some(pos) = header_buf[..header_len].windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = Some(pos + 4);
            break;
        }
        if header_len >= header_buf.len() {
            break;
        }
    }

    // Only parse the header portion as UTF-8, not the body data
    let header_end_pos = header_end.unwrap_or(header_len);
    let header_str = str::from_utf8(&header_buf[..header_end_pos]).map_err(|e| {
        println!("factory_http: UTF-8 parse error at byte {}, header_end={}", e.valid_up_to(), header_end_pos);
        HttpError::Parse
    })?;

    // Parse request line
    let first_line = header_str.lines().next().ok_or_else(|| {
        println!("factory_http: no lines in header");
        HttpError::Parse
    })?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next().ok_or_else(|| {
        println!("factory_http: no method in request line: {:?}", first_line);
        HttpError::Parse
    })?;
    let path = parts.next().ok_or_else(|| {
        println!("factory_http: no path in request line: {:?}", first_line);
        HttpError::Parse
    })?;

    println!("factory_http: {} {} (header_len={})", method, path, header_len);

    match (method, path) {
        ("GET", "/") => serve_html(socket).await,
        ("GET", "/config") => serve_config(socket, flash).await,
        ("POST", "/config") => handle_config_post(socket, &header_buf[..header_len], flash).await,
        ("POST", "/ota") => handle_ota_post(socket, &header_buf[..header_len], flash).await,
        _ => serve_404(socket).await,
    }
}

async fn serve_html<'a>(socket: &mut TcpSocket<'a>) -> Result<(), HttpError> {
    const HTML: &[u8] = myrtio_light_factory_page::FACTORY_PAGE_HTML_GZ;

    let mut header = heapless::String::<256>::new();
    let _ = write!(
        header,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        HTML.len()
    );
    socket.write_all(header.as_bytes()).await.map_err(|_| HttpError::Write)?;
    socket.flush().await.map_err(|_| HttpError::Write)?;
    
    for chunk in HTML.chunks(CHUNK_SIZE) {
        socket.write_all(chunk).await.map_err(|_| HttpError::Write)?;
        socket.flush().await.map_err(|_| HttpError::Write)?;
    }
    Ok(())
}

async fn serve_config<'a>(
    socket: &mut TcpSocket<'a>,
    flash: *mut FlashStorage<'static>,
) -> Result<(), HttpError> {
    let storage = AppPersistentStorage::new(flash, CONFIGURATION_PARTITION_OFFSET);
    
    let json = if let Some((_, _, config)) = storage.get_persistent_data() {
        format_config_json(&config)
    } else {
        let mut s = heapless::String::<512>::new();
        let _ = s.push_str("{}");
        s
    };

    let mut header = heapless::String::<256>::new();
    let _ = write!(
        header,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        json.len()
    );
    socket.write_all(header.as_bytes()).await.map_err(|_| HttpError::Write)?;
    socket.write_all(json.as_bytes()).await.map_err(|_| HttpError::Write)?;
    Ok(())
}

async fn handle_config_post<'a>(
    socket: &mut TcpSocket<'a>,
    header: &[u8],
    flash: *mut FlashStorage<'static>,
) -> Result<(), HttpError> {
    // Find where headers end
    let header_end = header.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(header.len());
    
    // Only parse header portion as UTF-8
    let header_str = str::from_utf8(&header[..header_end]).map_err(|_| HttpError::Parse)?;
    let content_length = parse_content_length(header_str).unwrap_or(0);

    let body_start = header_end;

    let mut body = heapless::Vec::<u8, 512>::new();
    
    // Copy any body already in header buffer
    if body_start < header.len() {
        let _ = body.extend_from_slice(&header[body_start..]);
    }

    // Read remaining body
    while body.len() < content_length {
        let mut buf = [0u8; 256];
        let n = socket.read(&mut buf).await.map_err(|_| HttpError::Read)?;
        if n == 0 {
            break;
        }
        let _ = body.extend_from_slice(&buf[..n]);
    }

    // Parse form data
    let body_str = str::from_utf8(&body).map_err(|_| HttpError::Parse)?;
    
    let mut storage = AppPersistentStorage::new(flash, CONFIGURATION_PARTITION_OFFSET);
    
    // Get current config or create default
    let mut config = storage.get_persistent_data()
        .map(|(_, _, c)| c)
        .unwrap_or_else(default_config);

    // Update config from form
    if let Err(e) = update_config_from_form(&mut config, body_str) {
        send_error_response(socket, 400, e).await?;
        return Ok(());
    }

    // Save config
    if storage.persist_device_config(config).is_none() {
        send_error_response(socket, 500, "Failed to save config").await?;
        return Ok(());
    }

    send_success_response(socket, "Configuration saved successfully").await?;
    Ok(())
}

async fn handle_ota_post<'a>(
    socket: &mut TcpSocket<'a>,
    header: &[u8],
    flash: *mut FlashStorage<'static>,
) -> Result<(), HttpError> {
    // Find where headers end (before binary body data)
    let header_end = header.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(header.len());
    
    // Only parse the header portion as UTF-8
    let header_str = str::from_utf8(&header[..header_end]).map_err(|e| {
        println!("factory_http: OTA header UTF-8 error at byte {}", e.valid_up_to());
        HttpError::Parse
    })?;
    let content_length = parse_content_length(header_str).ok_or_else(|| {
        println!("factory_http: OTA missing Content-Length");
        HttpError::Parse
    })?;

    println!("factory_http: OTA upload, size={} bytes", content_length);

    let body_start = header_end;

    // Initialize OTA updater
    let flash_ref = unsafe { &mut *flash };
    let mut part_buffer = [0u8; PARTITION_TABLE_MAX_LEN];
    
    let Ok(mut updater) = OtaUpdater::new(flash_ref, &mut part_buffer) else {
        send_error_response(socket, 500, "Failed to initialize OTA").await?;
        return Err(HttpError::Flash);
    };

    let Ok((mut partition, part_type)) = updater.next_partition() else {
        send_error_response(socket, 500, "No OTA partition available").await?;
        return Err(HttpError::Flash);
    };

    println!("factory_http: target partition {:?}", part_type);

    // Erase partition
    use embedded_storage::nor_flash::NorFlash;
    #[allow(clippy::cast_possible_truncation)]
    let part_capacity = partition.capacity() as u32;
    const ERASE_SECTOR: u32 = 4096;
    let erase_size = (content_length as u32)
        .saturating_add(ERASE_SECTOR - 1)
        / ERASE_SECTOR
        * ERASE_SECTOR;
    let erase_size = erase_size.min(part_capacity);

    println!("factory_http: erasing {} bytes...", erase_size);
    if partition.erase(0, erase_size).is_err() {
        send_error_response(socket, 500, "Failed to erase partition").await?;
        return Err(HttpError::Flash);
    }

    // Write firmware with alignment handling
    const ALIGN: usize = 4;
    let mut written: u32 = 0;
    let mut received: usize = 0;
    let mut tail = [0xFFu8; ALIGN];
    let mut tail_len: usize = 0;

    // Process any data already in header buffer
    if body_start < header.len() {
        let initial = &header[body_start..];
        received += initial.len();
        write_aligned_data(&mut partition, initial, &mut written, &mut tail, &mut tail_len)?;
    }

    // Read and write remaining data
    let mut chunk = [0u8; 1024];
    while received < content_length {
        let n = socket.read(&mut chunk).await.map_err(|_| HttpError::Read)?;
        if n == 0 {
            break;
        }
        received += n;
        write_aligned_data(&mut partition, &chunk[..n], &mut written, &mut tail, &mut tail_len)?;

        if received % 65536 < 1024 {
            println!("factory_http: progress {}/{} bytes", received, content_length);
        }
    }

    // Write final tail
    if tail_len > 0 {
        if partition.write(written, &tail).is_err() {
            return Err(HttpError::Flash);
        }
    }

    println!("factory_http: OTA complete, received {} bytes", received);

    // Activate partition
    if updater.activate_next_partition().is_err() || updater.set_current_ota_state(OtaImageState::New).is_err() {
        send_error_response(socket, 500, "Failed to activate partition").await?;
        return Err(HttpError::Flash);
    }

    send_success_response(socket, "OTA successful, rebooting...").await?;

    // Give time for response to be sent
    embassy_time::Timer::after(Duration::from_millis(500)).await;
    esp_hal::system::software_reset();
}

fn write_aligned_data<F: embedded_storage::nor_flash::NorFlash>(
    partition: &mut F,
    data: &[u8],
    written: &mut u32,
    tail: &mut [u8; 4],
    tail_len: &mut usize,
) -> Result<(), HttpError> {
    let mut idx = 0;

    // Complete partial word
    if *tail_len > 0 {
        let need = 4 - *tail_len;
        let take = need.min(data.len());
        tail[*tail_len..*tail_len + take].copy_from_slice(&data[..take]);
        *tail_len += take;
        idx += take;

        if *tail_len == 4 {
            partition.write(*written, tail).map_err(|_| HttpError::Flash)?;
            *written += 4;
            *tail_len = 0;
            tail.fill(0xFF);
        }
    }

    // Write aligned bulk
    let rem = &data[idx..];
    let aligned_len = rem.len() & !3;
    if aligned_len > 0 {
        partition.write(*written, &rem[..aligned_len]).map_err(|_| HttpError::Flash)?;
        *written += aligned_len as u32;
    }

    // Keep trailing bytes
    let tail_bytes = &rem[aligned_len..];
    if !tail_bytes.is_empty() {
        tail[..tail_bytes.len()].copy_from_slice(tail_bytes);
        *tail_len = tail_bytes.len();
    }

    Ok(())
}

async fn serve_404<'a>(socket: &mut TcpSocket<'a>) -> Result<(), HttpError> {
    let body = b"Not Found";
    let mut header = heapless::String::<128>::new();
    let _ = write!(header, "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n", body.len());
    socket.write_all(header.as_bytes()).await.map_err(|_| HttpError::Write)?;
    socket.write_all(body).await.map_err(|_| HttpError::Write)?;
    Ok(())
}

async fn send_error_response<'a>(socket: &mut TcpSocket<'a>, code: u16, message: &str) -> Result<(), HttpError> {
    let status = match code {
        400 => "400 Bad Request",
        500 => "500 Internal Server Error",
        _ => "500 Internal Server Error",
    };
    let mut response = heapless::String::<256>::new();
    let _ = write!(response, "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}", status, message.len(), message);
    socket.write_all(response.as_bytes()).await.map_err(|_| HttpError::Write)?;
    Ok(())
}

async fn send_success_response<'a>(socket: &mut TcpSocket<'a>, message: &str) -> Result<(), HttpError> {
    let mut response = heapless::String::<256>::new();
    let _ = write!(response, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", message.len(), message);
    socket.write_all(response.as_bytes()).await.map_err(|_| HttpError::Write)?;
    Ok(())
}

fn parse_content_length(header: &str) -> Option<usize> {
    for line in header.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("content-length:") {
            return line.split(':').nth(1)?.trim().parse().ok();
        }
    }
    None
}

fn format_config_json(config: &DeviceConfig) -> heapless::String<512> {
    let mut json = heapless::String::<512>::new();
    let _ = write!(json, "{{");
    let _ = write!(json, r#""wifi_ssid":"{}","#, config.wifi.ssid);
    let _ = write!(json, r#""wifi_password":"{}","#, config.wifi.password);
    let _ = write!(json, r#""mqtt_host":"{}","#, config.mqtt.host);
    let _ = write!(json, r#""mqtt_port":{},"#, config.mqtt.port);
    let _ = write!(json, r#""mqtt_username":"{}","#, config.mqtt.username);
    let _ = write!(json, r#""mqtt_password":"{}","#, config.mqtt.password);
    let _ = write!(json, r#""brightness_min":{},"#, config.light.brightness_min);
    let _ = write!(json, r#""brightness_max":{},"#, config.light.brightness_max);
    let _ = write!(json, r#""led_count":{},"#, config.light.led_count);
    let _ = write!(json, r#""skip_leds":{},"#, config.light.skip_leds);
    let _ = write!(json, "\"color_correction\":\"#{:06X}\"", config.light.color_correction);
    let _ = write!(json, "}}");
    json
}

fn default_config() -> DeviceConfig {
    DeviceConfig {
        wifi: WifiConfig {
            ssid: heapless::String::new(),
            password: heapless::String::new(),
        },
        mqtt: MqttConfig {
            host: heapless::String::new(),
            port: 1883,
            username: heapless::String::new(),
            password: heapless::String::new(),
        },
        light: LightConfig {
            brightness_min: 10,
            brightness_max: 255,
            led_count: 60,
            skip_leds: 0,
            color_correction: 0xFFFFFF,
        },
    }
}

fn update_config_from_form(config: &mut DeviceConfig, form: &str) -> Result<(), &'static str> {
    for pair in form.split('&') {
        let mut kv = pair.split('=');
        let key = kv.next().unwrap_or("");
        let value = kv.next().unwrap_or("");
        let value = url_decode(value);

        match key {
            "wifi_ssid" => {
                if value.len() > 32 {
                    return Err("WiFi SSID too long (max 32)");
                }
                config.wifi.ssid = heapless::String::try_from(value.as_str()).map_err(|_| "Invalid WiFi SSID")?;
            }
            "wifi_password" => {
                if value.len() > 64 {
                    return Err("WiFi password too long (max 64)");
                }
                config.wifi.password = heapless::String::try_from(value.as_str()).map_err(|_| "Invalid WiFi password")?;
            }
            "mqtt_host" => {
                if value.len() > 64 {
                    return Err("MQTT host too long (max 64)");
                }
                config.mqtt.host = heapless::String::try_from(value.as_str()).map_err(|_| "Invalid MQTT host")?;
            }
            "mqtt_port" => {
                config.mqtt.port = value.parse().map_err(|_| "Invalid MQTT port")?;
            }
            "mqtt_username" => {
                if value.len() > 32 {
                    return Err("MQTT username too long (max 32)");
                }
                config.mqtt.username = heapless::String::try_from(value.as_str()).map_err(|_| "Invalid MQTT username")?;
            }
            "mqtt_password" => {
                if value.len() > 64 {
                    return Err("MQTT password too long (max 64)");
                }
                config.mqtt.password = heapless::String::try_from(value.as_str()).map_err(|_| "Invalid MQTT password")?;
            }
            "brightness_min" => {
                config.light.brightness_min = value.parse().map_err(|_| "Invalid brightness_min")?;
            }
            "brightness_max" => {
                config.light.brightness_max = value.parse().map_err(|_| "Invalid brightness_max")?;
            }
            "led_count" => {
                config.light.led_count = value.parse().map_err(|_| "Invalid led_count")?;
            }
            "skip_leds" => {
                config.light.skip_leds = value.parse().map_err(|_| "Invalid skip_leds")?;
            }
            "color_correction" => {
                let hex = value.trim_start_matches('#').trim_start_matches("0x");
                config.light.color_correction = u32::from_str_radix(hex, 16).map_err(|_| "Invalid color_correction")?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn url_decode(input: &str) -> heapless::String<128> {
    let mut output = heapless::String::<128>::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '+' => { let _ = output.push(' '); }
            '%' => {
                let hex: heapless::String<2> = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    let _ = output.push(byte as char);
                }
            }
            _ => { let _ = output.push(c); }
        }
    }
    output
}
