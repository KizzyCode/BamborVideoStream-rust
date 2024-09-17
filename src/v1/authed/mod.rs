//! Authed API endpoints

pub mod p1;

use crate::{error::Error, services::config::Config};
use ehttpd::http::{Request, Response, ResponseExt};
use ehttpd_querystring::RequestQuerystringExt;
use sha2::{Digest, Sha256};
use std::{borrow::Cow, sync::Arc};

/// A ticket to assert a request is authed
pub struct AuthTicket {
    _private: (),
}
impl AuthTicket {
    /// Creates a new authed token
    pub(in crate::v1::authed) fn check(authtoken: &[u8], config: &Arc<Config>) -> Option<Self> {
        // Hash authtoken and validate hash
        let authdigest = format!("{:x}", Sha256::digest(authtoken));
        let true = authdigest == config.BAMBORVIDEOSTREAM_APIKEYSHA256 else {
            // Invalid auth token
            return None;
        };

        // Assert correct auth
        Some(Self { _private: () })
    }
}

/// Validates auth and calls the endpoint directly
pub fn call<T>(endpoint: T, request: Request, config: &Arc<Config>) -> Result<Response, Error>
where
    T: FnOnce(Request, &Arc<Config>, AuthTicket) -> Result<Response, Error>,
{
    /// The name of the authentication field
    const AUTH_FIELD: &[u8] = b"auth";
    /// The SHA-256 hash of an empty input
    const EMPTY: Cow<'_, [u8]> = Cow::Borrowed(b"");

    // Get querystring
    let Ok(querystring) = request.querystring() else {
        // Invalid query string
        return Ok(Response::new_400_badrequest());
    };

    // Validate auth token
    let authtoken = querystring.get(AUTH_FIELD).unwrap_or(&EMPTY);
    let Some(ticket) = AuthTicket::check(authtoken, config) else {
        // Invalid auth
        return Ok(Response::new_403_forbidden());
    };

    // Call endpoint
    endpoint(request, config, ticket)
}
