//! The server config

use crate::{error, error::Error};
use std::{
    borrow::Cow,
    env::{self, VarError},
};

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
    /// The *lowercase* SHA2-256 hash of the API key to use the server API
    ///
    /// # Discussion
    /// To disable the API key, use the SHA-256 hash of zero-length input (i.e.
    /// `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`)
    ///
    /// # Example
    /// A SHA2-256 hash of a randomly generated API key like
    /// `2b5025e892c82a2b65a5bc26cd96b68ac09e73d41e1523b479687e09ce01ddab`.
    pub BAMBORVIDEOSTREAM_APIKEYSHA256: String,
}
impl Config {
    /// Gets the config from the environment
    pub fn from_env() -> Result<Self, Error> {
        // Load config
        Ok(Config {
            BAMBORVIDEOSTREAM_SOCKADDR: Self::get_or("BAMBORVIDEOSTREAM_SOCKADDR", "[::]:80")?,
            BAMBORVIDEOSTREAM_CONNMAX: Self::get_or("BAMBORVIDEOSTREAM_CONNMAX", "1024")?.parse()?,
            BAMBORVIDEOSTREAM_APIKEYSHA256: Self::get("BAMBORVIDEOSTREAM_APIKEYSHA256")?,
        })
    }

    /// Gets the environment variable with the given name
    fn get(name: &str) -> Result<String, Error> {
        match env::var(name) {
            Ok(value) => Ok(value),
            Err(e) => Err(error!(with: e, r#"Missing required configuration environment variable "{name}""#)),
        }
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
