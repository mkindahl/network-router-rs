//! Common operations for both JSON and urlencoded data.

//use crate::web::Result;
//use hyper::{Body, Response, StatusCode};

static NOTFOUND: &[u8] = b"Not Found";

// Create a NOT FOUND response.
// pub(crate) fn resource_not_found() -> Result<Response<Body>> {
//     Ok(Response::builder()
//         .status(StatusCode::NOT_FOUND)
//         .body(NOTFOUND.into())
//         .unwrap())
// }

// pub(crate) fn bad_request(msg: String) -> Result<Response<Body>> {
//     Response::builder()
//         .status(StatusCode::BAD_REQUEST)
//         .header(header::CONTENT_TYPE, "text/html")
//         .body(Body::from(msg))
// }

// pub(crate) fn missing_header(header: &str) -> Result<Response<Body>> {
//     bad_request(format!(
//         "No {} header: use either application/json or application/x-www-form-urlencoded",
//         header
//     ))
// }

// pub(crate) fn missing_parameter(param: &str) -> Result<Response<Body>> {
//     bad_request(format!("Missing {}", param))
// }
