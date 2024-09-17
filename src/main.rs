#![doc = include_str!("../README.md")]
// Clippy lints
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unreachable)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::cognitive_complexity)]

mod error;
mod services;
mod v1;

use crate::{error::Error, services::config::Config};
use ehttpd::{
    http::{Request, Response, ResponseExt},
    Server,
};
use std::{process, sync::Arc};

/// Routes incoming requests
fn route(request: Request, config: &Arc<Config>) -> Response {
    // Route request
    let is_head = request.method == b"HEAD";
    let maybe_response: Result<Response, Error> = match (request.method.as_ref(), request.target.as_ref()) {
        // Authed endpoints
        (b"POST", target) if target.starts_with(b"/v1/p1") => {
            // Call endpoint via auth bridge
            v1::authed::call(v1::authed::p1::post, request, config)
        }

        // Site URLs
        (b"HEAD" | b"GET", target) if target.starts_with(b"/site/") => {
            // Call endpoint directly
            v1::site::handle(request)
        }

        // Fallback URLs
        (b"HEAD" | b"GET", b"/") => {
            // Redirect to main site URL
            Ok(Response::new_307_temporaryredirect(b"/site/app.html"))
        }
        _ => {
            // Deliver a good old 404
            Ok(Response::new_404_notfound())
        }
    };

    // Log server error and create appropriate response
    let mut response = maybe_response.unwrap_or_else(|error| {
        error.log();
        Response::new_500_internalservererror()
    });

    // Turn GET to HEAD if the request is a head request
    if is_head {
        // Transform into HEAD request
        response.make_head();
    }

    // Configure non-200 responses to explicitely close the associated connection
    if !response.status.starts_with(b"2") {
        response.set_content_length(0);
        response.set_connection_close();
    }
    response
}

/// A fallible main function
#[allow(clippy::unreachable, reason = "The server can only return in case of an error, but rust does not know this")]
fn try_main() -> Result<(), Error> {
    // Load config and init video services
    let config = Config::from_env()?;

    // Create server
    let config_ = Arc::new(config.clone());
    let server: Server<_> = Server::new(config.BAMBORVIDEOSTREAM_CONNMAX, move |source, sink| {
        // Route the request
        let config_ = config_.clone();
        ehttpd::reqresp(source, sink, move |request| route(request, &config_))
    });

    // Start the server and dispatch connections
    server.accept(config.BAMBORVIDEOSTREAM_SOCKADDR.as_ref())?;
    unreachable!("`server.accept` can only return in case of an error");
}

pub fn main() {
    if let Err(error) = try_main() {
        error.log();
        process::exit(1);
    }
}
