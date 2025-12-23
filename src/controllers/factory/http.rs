use embassy_time::{Duration, Timer};
use esp_storage::FlashStorage;
use heapless::String;

use super::CONFIGURATION_USECASES;
use crate::config::{self, DeviceConfig};
use crate::domain::dto::SystemInformation;
use crate::infrastructure::services::http::HttpResult;
use crate::infrastructure::services::http::connection::HttpConnection;

use crate::infrastructure::services::http::headers::{
    ContentEncoding, ContentHeaders, ContentType, HttpMethod, ResponseHeaders,
};
use crate::infrastructure::services::http::http_server::HttpHandler;
use crate::infrastructure::services::update_from_http;

#[derive(Default)]
pub struct FactoryHttpController {
    flash: *mut FlashStorage<'static>,
}

impl FactoryHttpController {
    pub fn new(flash: *mut FlashStorage<'static>) -> Self {
        Self { flash }
    }
}

impl HttpHandler for FactoryHttpController {
    async fn handle_request(&self, conn: HttpConnection<'_>) -> HttpResult {
        match conn.route() {
            (HttpMethod::Get, "/") => handle_get_html(conn).await,
            (HttpMethod::Get, "/api/system") => handle_get_system_information(conn).await,
            (HttpMethod::Get, "/api/configuration") => handle_get_configuration(conn).await,
            (HttpMethod::Post, "/api/configuration") => handle_set_configuration(conn).await,
            (HttpMethod::Post, "/api/ota") => handle_ota_update(conn, self.flash).await,
            _ => serve_404(conn).await,
        }
    }
}

async fn handle_get_html(mut conn: HttpConnection<'_>) -> HttpResult {
    const HTML: &[u8] = myrtio_light_factory_page::FACTORY_PAGE_HTML_GZ;
    let content = ContentHeaders::new_with_content_type(ContentType::TextHtml)
        .with_content_encoding(ContentEncoding::Gzip)
        .with_content_length(HTML.len());
    let headers = ResponseHeaders::success().with_content(content);
    conn.write_headers(&headers).await?;
    conn.write_body(HTML).await?;
    Ok(())
}

async fn handle_get_system_information(mut conn: HttpConnection<'_>) -> HttpResult {
    let mut build_version = String::<32>::new();
    build_version.push_str(config::BUILD_VERSION).unwrap();

    let system_information = SystemInformation {
        build_version,
        mac_address: config::mac_address(),
    };
    conn.write_json(&system_information).await
}

async fn handle_get_configuration(mut conn: HttpConnection<'_>) -> HttpResult {
    let config = CONFIGURATION_USECASES
        .lock(|cell| cell.borrow_mut().as_ref().unwrap().get_device_config())
        .unwrap();

    conn.write_json(&config).await
}

async fn handle_set_configuration(mut conn: HttpConnection<'_>) -> HttpResult {
    let config = conn.read_json::<DeviceConfig>().await?;
    let mut is_success = false;
    unsafe {
        CONFIGURATION_USECASES.lock_mut(|cell| {
            let usecases = cell.get_mut().as_mut().unwrap();
            is_success = usecases.set_device_config(&config).is_some();
        });
    }
    if is_success {
        conn.write_headers(&ResponseHeaders::success()).await?;
    } else {
        conn.write_headers(&ResponseHeaders::bad_request()).await?;
    }

    Ok(())
}

async fn handle_ota_update(
    mut conn: HttpConnection<'_>,
    flash: *mut FlashStorage<'static>,
) -> HttpResult {
    if let Err(_e) = update_from_http(&mut conn, flash).await {
        return conn.write_headers(&ResponseHeaders::internal_error()).await;
    }

    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;

    // Give time for response to be sent
    Timer::after(Duration::from_millis(500)).await;
    esp_hal::system::software_reset();
}

async fn serve_404(mut conn: HttpConnection<'_>) -> HttpResult {
    conn.write_headers(&ResponseHeaders::not_found()).await?;
    conn.write_body(b"Not Found").await
}
