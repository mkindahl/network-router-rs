//! Module that handles JSON requests.
//!
//! JSON structure contains the following fields:
//!
//!
use super::DatabaseRef;
use crate::{
    storage::{Database, Rule},
    web::{
        error::{Error, Result},
        resources::Resource,
    },
};
use ::http::uri::PathAndQuery;
use bytes::buf::ext::BufExt;
use futures::lock::Mutex;
use hyper::{header, Body, Method, Request, Response, StatusCode, Uri};
use std::sync::Arc;

/// Process a JSON request.
pub(crate) async fn process_request(
    database: DatabaseRef,
    resource: Resource,
    req: Request<Body>,
) -> Result<Response<Body>> {
    match req.method() {
        &Method::POST => process_post(database, resource, req).await,
        &Method::GET => process_get(database, resource).await,
        _ => Err(Error::ResourceNotFound),
    }
}

pub(crate) async fn process_post(
    database: DatabaseRef,
    resource: Resource,
    req: Request<Body>,
) -> Result<Response<Body>> {
    match resource {
        Resource::Rule(None) => process_add_rule_request(database, resource, req).await,
        Resource::Rule(Some(_rule)) => todo!(),
        Resource::Route(_rule, None) => todo!(),
        Resource::Route(_rule, Some(_route)) => todo!(),
    }
}

pub(crate) async fn process_get(
    database: DatabaseRef,
    resource: Resource,
) -> Result<Response<Body>> {
    let handle = database.lock().await;
    match resource {
        Resource::Rule(None) => Ok(Response::new(Body::from(serde_json::to_string(
            &handle.rules,
        )?))),
        Resource::Rule(Some(rule_no)) => match handle.rules.get(&rule_no) {
            Some(rule) => Ok(Response::new(Body::from(serde_json::to_string(rule)?))),
            None => Err(Error::ResourceNotFound),
        },
        Resource::Route(_, _) => todo!(),
    }
}

async fn process_add_rule_request(
    database: Arc<Mutex<Database>>,
    _resource: Resource,
    req: Request<Body>,
) -> Result<Response<Body>> {
    let authority = req.uri().authority().unwrap().to_owned();
    let scheme = req.uri().scheme().unwrap().to_owned();
    let whole_body = hyper::body::aggregate(req).await?;
    let rule: Rule = match serde_json::from_reader(whole_body.reader()) {
        Ok(rule) => rule,
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("error: {}", err)))
                .unwrap());
        }
    };
    let id = database.lock().await.add_rule(rule);
    let path_and_query: PathAndQuery = format!("/rules/{}", id).parse()?;
    let location = Uri::builder()
        .scheme(scheme.to_owned())
        .authority(authority)
        .path_and_query(path_and_query)
        .build()?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::LOCATION, location.to_string())
        .body(Body::from("{}"))?;
    Ok(response)
}
