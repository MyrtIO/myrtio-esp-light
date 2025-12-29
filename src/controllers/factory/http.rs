#![allow(clippy::await_holding_refcell_ref)]

#[cfg(feature = "log")]
use esp_println::println;
use heapless::String;

use super::CONFIGURATION_USECASES;
use crate::{
    config::{self, DeviceConfig},
    core::net::http::{
        ContentEncoding,
        ContentHeaders,
        ContentType,
        Error as HttpError,
        HttpConnection,
        HttpHandler,
        HttpMethod,
        HttpResult,
        ResponseHeaders,
        TextEncoding,
    },
    domain::dto::SystemInformation,
};

#[derive(Debug, Default)]
pub struct FactoryHttpController;

impl HttpHandler for FactoryHttpController {
    async fn handle_request(&self, conn: HttpConnection<'_>) -> HttpResult {
        let mut conn = conn;
        match conn.route() {
            (HttpMethod::Get, "/") => handle_get_html(&mut conn).await,
            (HttpMethod::Get, "/api/system") => {
                handle_get_system_information(&mut conn).await
            }
            (HttpMethod::Get, "/api/configuration") => {
                handle_get_configuration(&mut conn).await
            }
            (HttpMethod::Post, "/api/configuration") => {
                handle_set_configuration(&mut conn).await
            }
            (HttpMethod::Post, "/api/boot") => handle_boot(&mut conn).await,
            (HttpMethod::Post, "/api/ota") => handle_ota_update(&mut conn).await,
            _ => serve_404(&mut conn).await,
        }
    }
}

async fn handle_get_html(conn: &mut HttpConnection<'_>) -> HttpResult {
    const HTML: &[u8] = myrtio_light_factory_page::FACTORY_PAGE_HTML_GZ;
    let content = ContentHeaders::new(ContentType::TextHtml)
        .with_text_encoding(TextEncoding::Utf8)
        .with_encoding(ContentEncoding::Gzip)
        .with_length(HTML.len());
    let headers = ResponseHeaders::success().with_content(content);
    conn.write_headers(&headers).await?;
    conn.write_body(HTML).await?;
    Ok(())
}

async fn handle_get_system_information(conn: &mut HttpConnection<'_>) -> HttpResult {
    let mut build_version = String::<32>::new();
    build_version.push_str(config::BUILD_VERSION).unwrap();

    let system_information = SystemInformation {
        build_version,
        mac_address: config::mac_address(),
    };
    conn.write_json(&system_information).await
}

async fn handle_get_configuration(conn: &mut HttpConnection<'_>) -> HttpResult {
    let guard = CONFIGURATION_USECASES.lock().await;
    let config_usecases_ref = guard.borrow();
    let config_usecases = config_usecases_ref.as_ref().unwrap();
    let config = config_usecases.get_device_config().unwrap_or_default();

    conn.write_json(&config).await
}

async fn handle_set_configuration(conn: &mut HttpConnection<'_>) -> HttpResult {
    let config = conn.read_json::<DeviceConfig>().await?;
    let config_guard = CONFIGURATION_USECASES.lock().await;
    let mut usecases_ref = config_guard.borrow_mut();
    let usecases = usecases_ref.as_mut().unwrap();
    usecases
        .set_device_config(&config)
        .map_err(|_| HttpError::NoData)?;
    #[cfg(feature = "log")]
    println!("handle_set_configuration: configuration set");
    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;
    #[cfg(feature = "log")]
    println!("handle_set_configuration: headers written");
    Ok(())
}

async fn handle_boot(conn: &mut HttpConnection<'_>) -> HttpResult {
    let guard = super::FIRMWARE_USECASES.lock().await;
    let mut usecases_ref = guard.borrow_mut();
    let usecases = usecases_ref.as_mut().unwrap();
    usecases.boot_system().map_err(|_| HttpError::NoData)?;
    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;
    Ok(())
}

async fn handle_ota_update(conn: &mut HttpConnection<'_>) -> HttpResult {
    let guard = super::FIRMWARE_USECASES.lock().await;
    let usecases_ref = guard.borrow();
    let usecases = usecases_ref.as_ref().unwrap();
    usecases
        .update_firmware_from_http(conn)
        .await
        .map_err(|_| HttpError::NoData)?;
    // Note: No response needed - device will reboot after OTA update
    Ok(())
}

async fn serve_404(conn: &mut HttpConnection<'_>) -> HttpResult {
    conn.write_headers(&ResponseHeaders::not_found()).await?;
    conn.write_body(b"Not Found").await
}
