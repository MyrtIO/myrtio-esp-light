use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write as _;
#[cfg(feature = "log")]
use esp_println::println;
use heapless::{String, Vec};
use serde::{Serialize, de::DeserializeOwned};

use super::{
    Error,
    HttpResult,
    headers::{
        ContentHeaders,
        ContentType,
        HttpMethod,
        ResponseHeaders,
        TargetWriter as _,
        find_content_length,
        parse_request_line,
        read_heading,
    },
};

const HEADER_BUFFER_SIZE: usize = 512;
const BODY_BUFFER_SIZE: usize = 1024;
const BODY_RX_CHUNK_SIZE: usize = 256;
const STREAM_CHUNK_SIZE: usize = 1024;

/// A trait for reading chunks from a connection.
pub trait AsyncChunkedReader {
    fn content_length(&self) -> u32;
    fn read_and_then(
        &mut self,
        op: impl FnOnce(&[u8]),
    ) -> impl Future<Output = HttpResult>;
}

/// A trait for writing to a connection.
pub(crate) trait AsyncWriter {
    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = HttpResult>;
}

/// HTTP connection context
pub(crate) struct HttpConnection<'a> {
    pub method: HttpMethod,
    pub path: heapless::String<64>,

    socket: TcpSocket<'a>,
    content_length: u32,
    received: u32,
    header_end: usize,
    header_buf: Vec<u8, HEADER_BUFFER_SIZE>,
    body_buf: Vec<u8, BODY_BUFFER_SIZE>,
}

impl<'a> HttpConnection<'a> {
    /// Create a new HTTP connection from a socket.
    pub(crate) async fn from_socket(
        mut socket: TcpSocket<'a>,
    ) -> Result<Self, Error> {
        let mut header_buf = Vec::<u8, HEADER_BUFFER_SIZE>::new();
        for _ in 0..header_buf.capacity() {
            header_buf.push(0).unwrap();
        }
        let (header_end, header_len) =
            read_heading(header_buf.as_mut_slice(), &mut socket).await?;
        header_buf.truncate(header_len);

        // Only parse the headers portion (before body data) to avoid UB with binary
        // data
        let headers_only = &header_buf.as_slice()[..header_end];
        let header_str =
            core::str::from_utf8(headers_only).map_err(|_| Error::Parse)?;
        let (method, raw_path, rest_headers) =
            parse_request_line(header_str).ok_or(Error::Parse)?;
        let content_length = find_content_length(rest_headers).unwrap_or(0);

        let mut path = String::new();
        let _ = path.push_str(raw_path);
        Ok(Self {
            method,
            path,
            socket,
            header_buf,
            body_buf: Vec::new(),
            content_length,
            received: 0,
            header_end,
        })
    }

    /// Write the headers to the connection
    pub(crate) async fn write_headers(
        &mut self,
        headers: &ResponseHeaders,
    ) -> HttpResult {
        self.header_buf.clear();
        headers.write_to(&mut self.header_buf)?;
        self.write_header_buf().await
    }

    /// Write the body to the connection
    pub(crate) async fn write_body(&mut self, body: &[u8]) -> HttpResult {
        for chunk in body.chunks(STREAM_CHUNK_SIZE) {
            self.write_all(chunk).await?;
        }
        Ok(())
    }

    /// Write JSON to the connection
    ///
    /// Writes both headers and body.
    pub(crate) async fn write_json<T: Serialize>(&mut self, data: &T) -> HttpResult {
        for _ in 0..self.body_buf.capacity() {
            self.body_buf.push(0).unwrap();
        }
        let n = serde_json_core::to_slice(data, self.body_buf.as_mut_slice())
            .map_err(|_| Error::Closed)?;
        self.body_buf.truncate(n);
        let headers = ResponseHeaders::success()
            .with_content(ContentHeaders::new(ContentType::Json).with_length(n));

        self.write_headers(&headers).await?;

        self.write_body_buf().await?;
        Ok(())
    }

    /// Read JSON from the request body
    pub(crate) async fn read_json<T: DeserializeOwned>(
        &mut self,
    ) -> Result<T, Error> {
        let body = self.read_body().await?;
        let (data, _) = serde_json_core::from_slice(body).map_err(|_e| {
            #[cfg(feature = "log")]
            println!("factory_http: parse error: {:?}", _e);
            Error::Parse
        })?;
        Ok(data)
    }

    /// Get request method and path
    pub(crate) fn route(&self) -> (HttpMethod, &'_ str) {
        (self.method, self.path.as_str())
    }

    /// Write the body buffer to the connection
    async fn write_body_buf(&mut self) -> HttpResult {
        self.socket.write_all(self.body_buf.as_slice()).await?;
        self.socket.flush().await?;

        Ok(())
    }

    /// Write the header buffer to the connection
    async fn write_header_buf(&mut self) -> HttpResult {
        self.socket.write_all(self.header_buf.as_slice()).await?;
        self.socket.flush().await?;

        Ok(())
    }

    /// Read the request body
    async fn read_body(&mut self) -> Result<&[u8], Error> {
        if self.content_length == 0 {
            return Err(Error::NoData);
        }

        self.body_buf.clear();

        if self.header_buf.len() > self.header_end {
            let tail_len = self.header_buf.len() - self.header_end;
            for i in 0..tail_len {
                self.body_buf
                    .push(self.header_buf[self.header_end + i])
                    .unwrap();
            }
        }

        // Read remaining body
        while self.body_buf.len() < self.content_length as usize {
            let mut buf = [0u8; BODY_RX_CHUNK_SIZE];
            let n = self.socket.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            self.body_buf.extend_from_slice(&buf[..n]).unwrap();
        }

        Ok(&self.body_buf.as_slice()[..self.body_buf.len()])
    }
}

impl AsyncWriter for HttpConnection<'_> {
    async fn write_all(&mut self, buf: &[u8]) -> HttpResult {
        self.socket.write_all(buf).await?;
        self.socket.flush().await?;
        Ok(())
    }
}

impl AsyncChunkedReader for HttpConnection<'_> {
    fn content_length(&self) -> u32 {
        self.content_length
    }

    #[allow(clippy::cast_possible_truncation)]
    async fn read_and_then(&mut self, op: impl FnOnce(&[u8])) -> HttpResult {
        if self.content_length == 0 {
            esp_println::println!("http: content_length is 0, returning NoData");
            return Err(Error::NoData);
        }

        if self.received >= self.content_length {
            op(&[]);
            return Ok(());
        }
        self.body_buf.clear();

        if self.header_buf.len() > self.header_end {
            let trailer_len = self.header_buf.len() - self.header_end;
            for i in self.header_end..self.header_buf.len() {
                self.body_buf.push(self.header_buf[i]).unwrap();
            }
            self.header_buf.truncate(self.header_end);
            self.received += self.body_buf.len() as u32;
            esp_println::println!(
                "http: header trailer {} bytes, received={}",
                trailer_len,
                self.received
            );

            op(self.body_buf.as_slice());
            return Ok(());
        }

        for _ in 0..self.body_buf.capacity() - self.body_buf.len() {
            self.body_buf.push(0).unwrap();
        }

        let n = self.socket.read(self.body_buf.as_mut_slice()).await?;
        if n == 0 {
            esp_println::println!(
                "http: socket returned 0, received={}/{}",
                self.received,
                self.content_length
            );
            op(&[]);
            return Ok(());
        }
        self.received += n as u32;
        self.body_buf.truncate(n);
        op(self.body_buf.as_slice());

        Ok(())
    }
}
