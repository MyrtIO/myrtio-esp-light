use core::fmt::Write;

use embassy_net::tcp::{Error as TcpError, TcpSocket};

fn reason_phrase(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        409 => "Conflict",
        410 => "Gone",
        413 => "Request Entity Too Large",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

/// HTTP Content Encoding.
#[derive(Debug)]
pub enum ContentEncoding {
    Gzip,
}

impl ContentEncoding {
    /// Convert the content encoding to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentEncoding::Gzip => "gzip",
        }
    }
}

/// HTTP Content Type.
#[derive(Debug)]
pub enum ContentType {
    Json,
    TextHtml,
    TextPlain,
    ApplicationOctetStream,
}

/// Text Encoding.
#[derive(Debug)]
pub enum TextEncoding {
    Utf8,
}

impl TextEncoding {
    /// Convert the text encoding to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            TextEncoding::Utf8 => "utf-8",
        }
    }
}

impl ContentType {
    /// Convert the content type to a string.
    fn as_str(&self) -> &'static str {
        match self {
            ContentType::Json => "application/json",
            ContentType::TextHtml => "text/html",
            ContentType::TextPlain => "text/plain",
            ContentType::ApplicationOctetStream => "application/octet-stream",
        }
    }
}

/// HTTP socket connection policy.
#[derive(Debug)]
pub enum ConnectionPolicy {
    Close,
}

impl ConnectionPolicy {
    /// Convert the connection type to a string.
    fn as_str(&self) -> &'static str {
        match self {
            ConnectionPolicy::Close => "close",
        }
    }
}

pub trait BufferedWriter {
    fn write_to(&self, writer: &mut impl Write) -> Result<(), core::fmt::Error>;
}

/// HTTP Content Headers.
pub struct ContentHeaders {
    content_type: ContentType,
    content_encoding: Option<ContentEncoding>,
    content_length: Option<usize>,
    text_encoding: Option<TextEncoding>,
}

impl ContentHeaders {
    /// Create a new content headers with a content type.
    pub const fn new_with_content_type(content_type: ContentType) -> Self {
        Self {
            content_type,
            content_encoding: None,
            content_length: None,
            text_encoding: None,
        }
    }

    /// Set the content encoding.
    #[must_use]
    pub const fn with_content_encoding(mut self, content_encoding: ContentEncoding) -> Self {
        self.content_encoding = Some(content_encoding);
        self
    }

    /// Set the content length.
    #[must_use]
    pub const fn with_content_length(mut self, content_length: usize) -> Self {
        self.content_length = Some(content_length);
        self
    }

    /// Set the text encoding.
    #[must_use]
    pub const fn with_text_encoding(mut self, text_encoding: TextEncoding) -> Self {
        self.text_encoding = Some(text_encoding);
        self
    }
}

impl BufferedWriter for ContentHeaders {
    fn write_to(&self, writer: &mut impl Write) -> Result<(), core::fmt::Error> {
        write!(writer, "Content-Type: {}", self.content_type.as_str())?;
        if let Some(text_encoding) = &self.text_encoding {
            write!(writer, "; charset={}", text_encoding.as_str())?;
        }
        write!(writer, "\r\n")?;
        if let Some(content_encoding) = &self.content_encoding {
            write!(
                writer,
                "Content-Encoding: {}\r\n",
                content_encoding.as_str()
            )?;
        }
        if let Some(content_length) = self.content_length {
            write!(writer, "Content-Length: {}\r\n", content_length)?;
        }
        Ok(())
    }
}

/// Response Headers.
pub struct ResponseHeaders {
    status: u16,
    connection: ConnectionPolicy,
    content: Option<ContentHeaders>,
}

impl ResponseHeaders {
    /// Create empty response headers.
    pub const fn empty() -> Self {
        Self {
            status: 0,
            content: None,
            connection: ConnectionPolicy::Close,
        }
    }

    /// Create empty response headers with a status code.
    pub const fn from_code(code: u16) -> Self {
        Self::empty().with_code(code)
    }

    /// Set the success status code.
    pub const fn success() -> Self {
        Self::from_code(200)
    }

    /// Set the success no content status code.
    pub const fn success_no_content() -> Self {
        Self::from_code(204)
    }

    /// Set the not found status code.
    pub const fn not_found() -> Self {
        Self::from_code(404)
    }

    /// Set the internal server error status code.
    pub const fn internal_error() -> Self {
        Self::from_code(500)
    }

    /// Set the bad request status code.
    pub const fn bad_request() -> Self {
        Self::from_code(400)
    }

    /// Set the too large status code.
    pub const fn too_large() -> Self {
        Self::from_code(413)
    }

    /// Set the content headers.
    #[must_use]
    pub const fn with_content(mut self, content: ContentHeaders) -> Self {
        self.content = Some(content);
        self
    }

    /// Set the status code.
    #[must_use]
    pub const fn with_code(mut self, code: u16) -> Self {
        self.status = code;
        self
    }
}

impl BufferedWriter for ResponseHeaders {
    /// Write the response headers to a writer.
    fn write_to(&self, writer: &mut impl Write) -> Result<(), core::fmt::Error> {
        let reason = reason_phrase(self.status);
        write!(writer, "HTTP/1.1 {} {}\r\n", self.status, reason)?;
        if let Some(content) = &self.content {
            content.write_to(writer)?;
        }

        write!(writer, "Connection: {}\r\n", self.connection.as_str())?;
        write!(writer, "\r\n")?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    // Not supported for now
    // Trace,
    // Connect,
    // Upgrade,
}

impl HttpMethod {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "OPTIONS" => HttpMethod::Options,
            "HEAD" => HttpMethod::Head,
            _ => return None,
        })
    }
}

/// Parse the request line from the header string.
///
/// Returns the method, path, and rest of the header string.
pub(super) fn parse_request_line(header_str: &str) -> Option<(HttpMethod, &str, &str)> {
    let line_end = header_str.find("\r\n").unwrap_or(header_str.len());
    let first_line = &header_str[..line_end];
    let mut parts: core::str::SplitWhitespace<'_> = first_line.split_whitespace();
    let method = parts.next().and_then(HttpMethod::parse)?;
    let path = parts.next()?;

    Some((method, path, &header_str[line_end + 2..]))
}

/// Read the start line and headers from the socket.
///
/// Returns the position of the end of the headers and the length of the headers.
/// If the headers are not found, returns (0, 0).
pub(super) async fn read_heading(
    buf: &mut [u8],
    socket: &mut TcpSocket<'_>,
) -> Result<(usize, usize), TcpError> {
    let mut header_len = 0;
    let mut header_end = None;
    loop {
        let n = socket.read(&mut buf[header_len..]).await?;
        if n == 0 {
            return Ok((0, 0));
        }
        header_len += n;
        // Check for end of headers
        if let Some(pos) = buf[..header_len].windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = Some(pos + 4);
            break;
        }
        if header_len >= buf.len() {
            break;
        }
    }

    let header_end = header_end.unwrap_or(header_len);

    Ok((header_end, header_len))
}

/// Find the content length in the header string.
///
/// Returns the content length if found, otherwise None.
pub(super) fn find_content_length(header: &str) -> Option<u32> {
    const TARGET: &str = "content-length:";
    for line in header.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with(TARGET) {
            let length = line[TARGET.len()..].trim().parse::<u64>().ok()?;
            if length > u32::MAX as u64 {
                return None;
            }
            return Some(length as u32);
        }
    }
    None
}
