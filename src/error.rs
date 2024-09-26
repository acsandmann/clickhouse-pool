use clickhouse::error::Error as ClickhouseError;
use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};
use tokio::sync::AcquireError;

/// Represents errors that can occur in the connection pool.
#[derive(Debug)]
pub enum Error {
    /// Error occurred while acquiring a semaphore permit.
    AcquireError,
    /// Error occurred while creating or connecting the ClickHouse client.
    Clickhouse(ClickhouseError),
    /// An unknown error occurred.
    Unknown,
}

impl From<ClickhouseError> for Error {
    fn from(err: ClickhouseError) -> Self {
        Error::Clickhouse(err)
    }
}

impl From<AcquireError> for Error {
    fn from(_err: AcquireError) -> Self {
        Error::AcquireError
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::AcquireError => write!(f, "Failed to acquire a semaphore permit"),
            Error::Clickhouse(err) => write!(f, "ClickHouse error: {}", err),
            Error::Unknown => write!(f, "An unknown error occurred"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Clickhouse(err) => Some(err),
            _ => None,
        }
    }
}
