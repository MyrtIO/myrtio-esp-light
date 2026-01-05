#![allow(clippy::await_holding_refcell_ref)]

#[cfg(feature = "log")]
use esp_println::println;
use heapless::String;
use serde::{Deserialize, Serialize};

use super::{CONFIGURATION_USECASES, LIGHT_STATE_SERVICE};
use crate::{
    config::{
        self,
        ColorOrder,
        DeviceConfig,
        LightConfig,
        MqttConfig,
        WifiConfig,
        pack_color_correction,
        unpack_color_correction_rgb24,
        unpack_color_order,
    },
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
    domain::{dto::SystemInformation, ports::LightStateChanger},
};

// ============================================================================
// HTTP API DTOs
// These expose color_order as a separate field while the internal config
// packs it into the high byte of color_correction.
// ============================================================================

/// Light configuration as exposed via HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightConfigApi {
    pub brightness_min: u8,
    pub brightness_max: u8,
    pub led_count: u8,
    pub skip_leds: u8,
    /// RGB24 color correction (0xRRGGBB format)
    pub color_correction: u32,
    /// LED color channel order
    pub color_order: ColorOrder,
}

impl From<LightConfig> for LightConfigApi {
    fn from(config: LightConfig) -> Self {
        Self {
            brightness_min: config.brightness_min,
            brightness_max: config.brightness_max,
            led_count: config.led_count,
            skip_leds: config.skip_leds,
            color_correction: unpack_color_correction_rgb24(config.color_correction),
            color_order: unpack_color_order(config.color_correction),
        }
    }
}

impl From<LightConfigApi> for LightConfig {
    fn from(api: LightConfigApi) -> Self {
        Self {
            brightness_min: api.brightness_min,
            brightness_max: api.brightness_max,
            led_count: api.led_count,
            skip_leds: api.skip_leds,
            color_correction: pack_color_correction(
                api.color_order,
                api.color_correction,
            ),
        }
    }
}

/// Device configuration as exposed via HTTP API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigApi {
    pub wifi: WifiConfig,
    pub mqtt: MqttConfig,
    pub light: LightConfigApi,
}

impl From<DeviceConfig> for DeviceConfigApi {
    fn from(config: DeviceConfig) -> Self {
        Self {
            wifi: config.wifi,
            mqtt: config.mqtt,
            light: config.light.into(),
        }
    }
}

impl From<DeviceConfigApi> for DeviceConfig {
    fn from(api: DeviceConfigApi) -> Self {
        Self {
            wifi: api.wifi,
            mqtt: api.mqtt,
            light: api.light.into(),
        }
    }
}

/// Request to test LED color output.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightTestRequest {
    /// Red channel (0-255)
    pub r: u8,
    /// Green channel (0-255)
    pub g: u8,
    /// Blue channel (0-255)
    pub b: u8,
    /// Brightness (0-255)
    pub brightness: u8,
}

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
            (HttpMethod::Post, "/api/configuration/light") => {
                handle_set_light_config(&mut conn).await
            }
            (HttpMethod::Post, "/api/light/test") => {
                handle_light_test(&mut conn).await
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

    // Convert to API format (unpacks color_order from color_correction)
    let api_config: DeviceConfigApi = config.into();
    conn.write_json(&api_config).await
}

async fn handle_set_configuration(conn: &mut HttpConnection<'_>) -> HttpResult {
    // Read API format and convert to internal (packs color_order into
    // color_correction)
    let api_config = conn.read_json::<DeviceConfigApi>().await?;
    let config: DeviceConfig = api_config.into();

    let config_guard = CONFIGURATION_USECASES.lock().await;
    let mut usecases_ref = config_guard.borrow_mut();
    let usecases = usecases_ref.as_mut().unwrap();
    usecases
        .save_device_config(&config)
        .map_err(|_| HttpError::NoData)?;
    #[cfg(feature = "log")]
    println!("handle_set_configuration: configuration set");
    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;
    #[cfg(feature = "log")]
    println!("handle_set_configuration: headers written");
    Ok(())
}

async fn handle_set_light_config(conn: &mut HttpConnection<'_>) -> HttpResult {
    // Read API format and convert to internal (packs color_order into
    // color_correction)
    let api_config = conn.read_json::<LightConfigApi>().await?;
    let config: LightConfig = api_config.into();

    let config_guard = CONFIGURATION_USECASES.lock().await;
    let mut usecases_ref = config_guard.borrow_mut();
    let usecases = usecases_ref.as_mut().unwrap();
    usecases
        .set_light_config(&config)
        .map_err(|_| HttpError::NoData)?;
    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;
    Ok(())
}

async fn handle_light_test(conn: &mut HttpConnection<'_>) -> HttpResult {
    use crate::domain::dto::LightChangeIntent;

    let request = conn.read_json::<LightTestRequest>().await?;

    // Build intent to set power on, brightness, and color
    let intent = LightChangeIntent::new()
        .with_power(true)
        .with_brightness(request.brightness)
        .with_color(request.r, request.g, request.b);

    let guard = LIGHT_STATE_SERVICE.lock().await;
    let light_ref = guard.borrow();
    let light = light_ref.as_ref().ok_or(HttpError::NoData)?;
    light
        .apply_light_intent(intent)
        .map_err(|_| HttpError::NoData)?;

    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;
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
    let mut usecases_ref = guard.borrow_mut();
    let usecases = usecases_ref.as_mut().unwrap();
    usecases
        .update_firmware_from_http(conn)
        .await
        .map_err(|_| HttpError::NoData)?;
    conn.write_headers(&ResponseHeaders::success_no_content())
        .await?;
    usecases.boot_system().unwrap();
    Ok(())
}

async fn serve_404(conn: &mut HttpConnection<'_>) -> HttpResult {
    conn.write_headers(&ResponseHeaders::not_found()).await?;
    conn.write_body(b"Not Found").await
}
