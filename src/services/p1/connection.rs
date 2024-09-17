//! A TLS connection to a P1 device

use crate::{error, error::Error};
use native_tls::{Protocol, TlsConnector, TlsStream};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

/// A TLS connection to a P1 device
#[derive(Debug)]
pub struct P1Connection {
    /// The TLS connection
    connection: TlsStream<TcpStream>,
}
impl P1Connection {
    /// The default timeout
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

    /// Creates a new connection to a P1 device
    pub fn new(address: &str) -> Result<Self, Error> {
        // Connect to the device
        let connection = TcpStream::connect(address)?;
        connection.set_read_timeout(Some(Self::DEFAULT_TIMEOUT))?;
        connection.set_write_timeout(Some(Self::DEFAULT_TIMEOUT))?;

        // Create a TLS stream from the TCP connection
        let tls = TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .min_protocol_version(Some(Protocol::Tlsv12))
            .build()?;
        let connection = tls.connect(address, connection)?;

        // Init self
        Ok(Self { connection })
    }

    /// Performs a login to the device to get a session
    pub fn login(mut self, pin: &str) -> Result<P1Session, Error> {
        /// The login packet template
        const LOGIN_PACKET: [u8; 80] = [
            // v1
            0x40, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, // 8
            // v2
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8
            // Username
            b'b', b'b', b'l', b'p', 0x00, 0x00, 0x00, 0x00, // 8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 16
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 24
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 32
            // PIN
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 16
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 24
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 32
        ];
        /// The offset of the PIN field in the login packet
        const LOGIN_PACKET_PIN: usize = 48;

        // Validate PIN length
        let _pin_len @ ..32 = pin.len() else {
            // Reject invalid PIN
            return Err(error!("Invalid PIN length: {}", pin.len()));
        };

        // Assemble login packet
        let mut packet = LOGIN_PACKET;
        #[allow(clippy::indexing_slicing, reason = "Offset is always valid and PIN length is checked")]
        packet[LOGIN_PACKET_PIN..][..pin.len()].copy_from_slice(pin.as_bytes());

        // Send login packet
        self.connection.write_all(&packet)?;
        Ok(P1Session { connection: self.connection })
    }
}

/// An authenticated session to a P1 device
#[derive(Debug)]
pub struct P1Session {
    /// The TLS connection
    connection: TlsStream<TcpStream>,
}
impl P1Session {
    /// Receives a JPEG image from the device
    pub fn jpeg(&mut self) -> Result<Vec<u8>, Error> {
        // Read JPEG size
        let mut size = [0; 4];
        self.connection.read_exact(&mut size)?;
        let size = u32::from_le_bytes(size);

        // Read 12 bytes version stuff
        let mut version = [0; 12];
        self.connection.read_exact(&mut version)?;

        // Read JPEG image
        let mut jpeg = vec![0; size as usize];
        self.connection.read_exact(&mut jpeg)?;
        Ok(jpeg)
    }
}
