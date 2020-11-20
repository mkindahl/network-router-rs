//! Module with common internal methods. These are all using the
//! web-internal Result and Error types.

use super::error::{Error, Result};
use hyper::{
    header::{self, HeaderName, HeaderValue},
    Body, Method, Request,
};

fn to_str<'a>(header: HeaderName, value: Result<&'a HeaderValue>) -> Result<&'a str> {
    match value {
        Ok(value) => value.to_str().map_err(|_| Error::BadHeader(header)),
        Err(err) => Err(err),
    }
}

/// Get the content type of the message.
///
/// For POST requests, the content type is given by the "Content-Type"
/// header, but for a GET request the type is given by the "Accept"
/// header.
pub(crate) fn get_content_type(req: &Request<Body>) -> Result<&str> {
    let headers = req.headers();
    match req.method() {
        &Method::POST => to_str(
            header::CONTENT_TYPE,
            headers
                .get(header::CONTENT_TYPE)
                .ok_or(Error::MissingHeader(header::CONTENT_TYPE)),
        ),

        &Method::GET => to_str(
            header::ACCEPT,
            headers
                .get(header::ACCEPT)
                .ok_or(Error::MissingHeader(header::ACCEPT)),
        ),
        _ => todo!(),
    }
}
