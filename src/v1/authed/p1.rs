//! Gets the last JPEG for the given P1 device

use crate::{
    error::Error,
    services::{config::Config, p1::P1Service},
    v1::authed::AuthTicket,
};
use core::str;
use ehttpd::http::{Request, Response, ResponseExt};
use ehttpd_querystring::{querystringext::QueryStringExt, RequestQuerystringExt};
use std::sync::{Arc, Weak};

/// Gets the service for the given P1 device
fn image_service(address: &str, pin: &str) -> Arc<P1Service> {
    // Get the associated device service
    #[allow(clippy::expect_used, reason = "Locking a mutex should never fail under normal conditions")]
    let mut services = P1Service::services().lock().expect("Failed to lock services registry");

    // Try to get a living service for the given device
    let maybe_service = services.get(address).and_then(Weak::upgrade);
    if let Some(service) = maybe_service {
        // The service is still alive, use it
        service
    } else {
        // Create new service and get a weak reference for the registry
        let service = P1Service::new(address, pin);
        let service_weak = Arc::downgrade(&service);

        // Register the weak reference and return the service
        services.insert(address.to_string(), service_weak);
        service
    }
}

/// Gets the last JPEG for the given P1 device
pub fn post(request: Request, _: &Arc<Config>, _: AuthTicket) -> Result<Response, Error> {
    /// The name of the device address field
    const DEVICEADDRESS_FIELD: &[u8] = b"address";
    /// The name of the device PIN field
    const DEVICEPIN_FIELD: &[u8] = b"pin";

    // Get the query string
    let Ok(querystring) = request.querystring() else {
        // The query string was invalid
        return Ok(Response::new_400_badrequest());
    };

    // Get the device name and secret
    let Ok(Some(address)) = querystring.get_str(DEVICEADDRESS_FIELD) else {
        // The device address is missing
        return Ok(Response::new_400_badrequest());
    };
    let Ok(Some(pin)) = querystring.get_str(DEVICEPIN_FIELD) else {
        // The device PIN is missing
        return Ok(Response::new_400_badrequest());
    };

    // Get the image
    let mut response = Response::new_200_ok();
    let service = image_service(address, pin);
    if let Some(image) = service.jpeg() {
        // Set the image as body
        response.set_body_data(image);
        response.set_content_type("image/jpeg");
    } else {
        // Set text/plain
        response.set_content_type("text/plain");
    }
    Ok(response)
}
