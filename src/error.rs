//! Implements the crate's error type

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    fmt::{self, Debug, Display, Formatter},
};

/// Creates a new error
#[macro_export]
macro_rules! error {
    (with: $error:expr, $($arg:tt)*) => {{
        let error = format!($($arg)*);
        let source: Box<dyn std::error::Error + Send> = Box::new($error);
        $crate::error::Error::new(error, Some(source))
    }};
    ($($arg:tt)*) => {{
        let error = format!($($arg)*);
        $crate::error::Error::new(error, None)
    }};
}

/// The crates error type
#[derive(Debug)]
pub struct Error {
    /// The error description
    pub error: String,
    /// The underlying error
    pub source: Option<Box<dyn std::error::Error + Send>>,
    /// The backtrace
    pub backtrace: Backtrace,
}
impl Error {
    /// Creates a new error
    #[doc(hidden)]
    pub fn new(error: String, source: Option<Box<dyn std::error::Error + Send>>) -> Self {
        let backtrace = Backtrace::capture();
        Self { error, source, backtrace }
    }

    /// Whether the error has captured a backtrace or not
    pub fn has_backtrace(&self) -> bool {
        self.backtrace.status() == BacktraceStatus::Captured
    }

    /// Logs `self` to stderr
    pub fn log(&self) {
        // Print error
        eprintln!("{self}");
        if self.has_backtrace() {
            // Print backtrace if available
            eprintln!("{}", self.backtrace)
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Print the error
        writeln!(f, "{}", self.error)?;

        // Print the source
        if let Some(source) = &self.source {
            writeln!(f, " caused by: {}", source)?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let error: &(dyn std::error::Error + 'static) = self.source.as_deref()?;
        Some(error)
    }
}
impl From<ehttpd::error::Error> for Error {
    fn from(error: ehttpd::error::Error) -> Self {
        error!(with: error, "HTTP server error")
    }
}
impl From<native_tls::Error> for Error {
    fn from(error: native_tls::Error) -> Self {
        error!(with: error, "TLS error")
    }
}
impl<T> From<native_tls::HandshakeError<T>> for Error
where
    T: Debug + Send + 'static,
{
    fn from(error: native_tls::HandshakeError<T>) -> Self {
        error!(with: error, "TLS handshake error")
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        error!(with: error, "Parsing error")
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        error!(with: error, "UTF-8 error")
    }
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        error!(with: error, "In/out error")
    }
}
