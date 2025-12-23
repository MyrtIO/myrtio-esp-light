pub mod connection;
pub mod headers;
pub mod http_server;

#[derive(Debug)]
pub enum HttpError {
    Read,
    Closed,
    WriteHeaders,
    Parse,
    NoData,
    FormatHeaders,
}

impl From<core::fmt::Error> for HttpError {
    fn from(_error: core::fmt::Error) -> Self {
        HttpError::FormatHeaders
    }
}

impl From<embassy_net::tcp::Error> for HttpError {
    fn from(err: embassy_net::tcp::Error) -> Self {
        match err {
            embassy_net::tcp::Error::ConnectionReset => HttpError::Closed,
        }
    }
}

pub type HttpResult = Result<(), HttpError>;
