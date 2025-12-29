pub(crate) mod connection;
pub(crate) mod headers;
pub(crate) mod server;

pub(crate) use connection::{AsyncChunkedReader, HttpConnection};
pub(crate) use headers::{
    ContentEncoding,
    ContentHeaders,
    ContentType,
    ResponseHeaders,
    TextEncoding,
};
pub(crate) use headers::HttpMethod;
pub(crate) use server::HttpHandler;
pub(crate) use server::HttpServer;

#[derive(Debug)]
pub enum Error {
    Closed,
    Parse,
    NoData,
    FormatHeaders,
}

impl From<core::fmt::Error> for Error {
    fn from(_error: core::fmt::Error) -> Self {
        Error::FormatHeaders
    }
}

impl From<embassy_net::tcp::Error> for Error {
    fn from(err: embassy_net::tcp::Error) -> Self {
        match err {
            embassy_net::tcp::Error::ConnectionReset => Error::Closed,
        }
    }
}

pub type HttpResult = Result<(), Error>;
