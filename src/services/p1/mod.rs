//! An image service for a P1S/P1P client

mod connection;

use crate::{error::Error, services::p1::connection::P1Connection};
use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock, Mutex, Weak},
    thread,
    time::Duration,
};

/// A service for a P1S/P1P client
#[derive(Debug)]
pub struct P1Service {
    /// The last image
    last_image: Mutex<Option<Vec<u8>>>,
}
impl P1Service {
    /// The duration of a single frame
    const FRAME_DURATION: Duration = Duration::from_secs(1);
    /// Keep alive for 600 frames (aka 10 minutes)
    const KEEP_ALIVE: usize = 600;

    /// Starts a new P1 service
    pub fn new(address: &str, pin: &str) -> Arc<Self> {
        // Setup service state
        let service = Arc::new(Self { last_image: Mutex::new(None) });

        // Start runloop thread
        let address_ = address.to_string();
        let pin_ = pin.to_string();
        let service_ = service.clone();
        thread::spawn(|| Self::runloop(address_, pin_, service_));

        // Return the service
        service
    }

    /// Gets the last JPEG of the connected device
    pub fn jpeg(&self) -> Option<Vec<u8>> {
        // Get last image
        #[allow(clippy::expect_used, reason = "Locking a mutex should never fail under normal conditions")]
        let last_image_lock = self.last_image.lock().expect("Failed to lock mutex");
        last_image_lock.clone()
    }

    /// The globally registered P1 services
    ///
    /// # Note
    /// The registry only stores weak references, because if the associated runloop is dead, the service is dead too.
    pub fn services() -> &'static Mutex<BTreeMap<String, Weak<P1Service>>> {
        static IMAGE_SERVICES: LazyLock<Mutex<BTreeMap<String, Weak<P1Service>>>> =
            LazyLock::new(|| Mutex::new(BTreeMap::new()));
        &IMAGE_SERVICES
    }

    /// The service runloop
    #[allow(clippy::expect_used, reason = "We run in a separate thread and may panick")]
    fn runloop(address: String, pin: String, service: Arc<Self>) {
        // Fallible runloop scope
        let try_catch = || -> Result<(), Error> {
            // Setup connection
            let connection = P1Connection::new(&address)?;
            let mut session = connection.login(&pin)?;

            // Fetch some images for some time
            for _ in 0..Self::KEEP_ALIVE {
                // Set JPEG
                let jpeg = session.jpeg()?;
                let mut last_image = service.last_image.lock().expect("Failed to lock mutex");
                *last_image = Some(jpeg);

                // Unlock shared state and pause for the frame duration
                drop(last_image);
                thread::sleep(Self::FRAME_DURATION);
            }

            // Keep-alive expired
            Ok(())
        };

        // Run fallible code
        try_catch().expect("Image service terminated");
    }
}
