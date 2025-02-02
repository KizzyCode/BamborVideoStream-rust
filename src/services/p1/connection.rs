//! A TLS connection to a P1 device

use crate::error;
use crate::error::Error;
use crate::services::p1::tls::DangerousAcceptAnyCert;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

/// A TLS connection to a P1 device
#[derive(Debug)]
pub struct P1Connection {
    /// The TLS connection
    connection: StreamOwned<ClientConnection, TcpStream>,
}
impl P1Connection {
    /// The default timeout
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

    /// Creates a new connection to a P1 device
    pub fn new(address: &str) -> Result<Self, Error> {
        // Create TLS config
        let dangerous_accept_any_cert = Arc::new(DangerousAcceptAnyCert);
        let tls_config = (ClientConfig::builder().dangerous())
            .with_custom_certificate_verifier(dangerous_accept_any_cert)
            .with_no_client_auth();

        // Connect to the device
        let tcp = TcpStream::connect(address)?;
        tcp.set_read_timeout(Some(Self::DEFAULT_TIMEOUT))?;
        tcp.set_write_timeout(Some(Self::DEFAULT_TIMEOUT))?;

        // Connect to P1S
        let tls_config = Arc::new(tls_config);
        let address = tcp.peer_addr().map(|a| a.ip()).map(ServerName::from)?;
        let tls = ClientConnection::new(tls_config, address)?;

        // Init self
        let connection = StreamOwned::new(tls, tcp);
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
    connection: StreamOwned<ClientConnection, TcpStream>,
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
