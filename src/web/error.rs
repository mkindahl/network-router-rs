use hyper::{self, header::HeaderName, Body, Response, StatusCode};

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
    ResourceNotFound,
    BadMethod(String),
    MissingParameter(String),
    BadParameter(String),
    MissingHeader(HeaderName),
    BadHeader(HeaderName),
    Internal(String),
    UnsatisfiedInvariant,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ResourceNotFound => write!(f, "resource not found"),
            Error::BadMethod(method) => write!(f, "bad method {}", method),
            Error::MissingParameter(_) => write!(f, "missing parameter"),
            Error::BadParameter(param) => write!(f, "bad parameter: {}", param),
            Error::MissingHeader(_) => write!(f, "missing header"),
            Error::BadHeader(header) => write!(f, "bad header: {}", header),
            Error::Internal(text) => write!(f, "internal error: {}", text),
            Error::UnsatisfiedInvariant => write!(f, "invariant error"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::ResourceNotFound => "resource not found",
            Error::BadMethod(_) => "bad method",
            Error::MissingParameter(_) => "missing parameter",
            Error::BadParameter(_) => "bad parameter",
            Error::MissingHeader(_) => "missing header",
            Error::BadHeader(_) => "bad header",
            Error::Internal(_) => "internal error",
            Error::UnsatisfiedInvariant => "unsatisfied invariant",
        }
    }
}

impl From<Error> for Response<Body> {
    fn from(err: Error) -> Self {
        let builder = Response::builder();
        match err {
            Error::ResourceNotFound => builder
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("resource not found".to_string()))
                .unwrap(),
            err => builder
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("{}", err)))
                .unwrap(),
        }
    }
}

impl From<askama::Error> for Error {
    fn from(err: askama::Error) -> Self {
        Error::Internal(format!("{}", err))
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Internal(format!("{}", err))
    }
}

impl From<hyper::http::Error> for Error {
    fn from(err: hyper::http::Error) -> Self {
        Error::Internal(format!("{}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(format!("{}", err))
    }
}

impl From<::http::uri::InvalidUri> for Error {
    fn from(err: ::http::uri::InvalidUri) -> Self {
        Error::Internal(format!("{}", err))
    }
}
