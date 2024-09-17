//! Delivers the static website

use crate::error::Error;
use ehttpd::http::{Request, Response, ResponseExt};

/// The sitemap to map an URL to the content-type and content
#[inline]
const fn sitemap(url: &[u8]) -> Option<(&'static [u8], &'static [u8])> {
    match url {
        b"/site/app.html" => Some((b"text/html", include_bytes!("../../site/app.html"))),
        b"/site/p1.html" => Some((b"text/html", include_bytes!("../../site/p1.html"))),
        b"/site/p1.js" => Some((b"application/javascript", include_bytes!("../../site/p1.js"))),
        b"/site/loading.js" => Some((b"application/javascript", include_bytes!("../../site/loading.js"))),
        _ => None,
    }
}

/// Handles the request
pub fn handle(request: Request) -> Result<Response, Error> {
    // Resolve the website
    let Some((content_type, content)) = sitemap(&request.target) else {
        // No such site
        return Ok(Response::new_404_notfound());
    };

    // Return the site
    let mut response = Response::new_200_ok();
    response.set_content_type(content_type);
    response.set_body_data(content);
    Ok(response)
}
