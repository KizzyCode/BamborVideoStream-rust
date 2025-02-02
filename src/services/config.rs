//! The server config

use crate::error;
use crate::error::Error;
use std::borrow::Cow;
use std::env::{self, VarError};

/// The server config
#[derive(Debug, Clone)]
#[allow(non_snake_case, reason = "We want to map the exact naming of the environment variables")]
pub struct Config {
    /// The socket address to listen on
    ///
    /// # Example
    /// An `address:port` combination; defaults to `[::]:80` to listen on all local IP addresses on port 80
    pub BAMBORVIDEOSTREAM_SOCKADDR: Cow<'static, str>,
    /// The maximum amount of open connections
    ///
    /// # Discussion
    /// Each opened connection requires at least one separate thread; depending on your OS and environment this may
    /// cause significant load. The default is `1024` – this should probably be increased for prod servers.
    pub BAMBORVIDEOSTREAM_CONNMAX: usize,
}
impl Config {
    /// Gets the config from the environment
    pub fn from_env() -> Result<Self, Error> {
        // Load config
        Ok(Config {
            BAMBORVIDEOSTREAM_SOCKADDR: Self::get_or("BAMBORVIDEOSTREAM_SOCKADDR", "[::]:80")?,
            BAMBORVIDEOSTREAM_CONNMAX: Self::get_or("BAMBORVIDEOSTREAM_CONNMAX", "1024")?.parse()?,
        })
    }

    /// Gets the environment variable with the given name or returns the default value
    fn get_or(name: &str, default: &'static str) -> Result<Cow<'static, str>, Error> {
        match env::var(name) {
            Ok(value) => Ok(Cow::Owned(value)),
            Err(VarError::NotPresent) => Ok(Cow::Borrowed(default)),
            Err(e) => Err(error!(with: e, r#"Invalid configuration environment variable "{name}""#)),
        }
    }
}
