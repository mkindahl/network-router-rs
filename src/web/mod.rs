//! Web interface module.
//!
//! There are rules that decide what port to listen on and what ports
//! to use for the outgoing connections or packages.
//!
//! # Resources
//!
//! Resources are referenced in the same way independent of the operation on it.
//!
//! * `/rules`: All rules of the router.
//! * `/rules/123`: Specific rule on the router
//! * `/rules/123/routes`: All routes for a specific rule
//! * `/rules/123/routes/1`: A specific route for a specific rule
//!
//! GET is used to read information.
//!
//! POST is used to add objects.
//!
//! Since the standard only support POST and GET for forms, the
//! parameter `action` will be used to indicate the operation for
//! urlencoded data, but all operations are supported with JSON.
//!
//! * `/rules`: Used when adding a new rule. Will return a reference
//!   to the newly added rule.
//!
//! * `/rules/123`: Direct rule references will be used when modifying
//!   or deleting a rule.
//!
//! * `/rules/123/routes/1`: Direct reference to a specific route. It
//!   is only possible to delete (shutdown) a route, not modify an
//!   existing route.

mod common;
mod error;
mod form;
mod json;
mod resources;

use crate::{
    storage::Database,
    web::{common::get_content_type, resources::Resource},
};
use futures::lock::Mutex;
use hyper::{
    header,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Result, Server, StatusCode,
};
use log::{error, info};
use std::{str::FromStr, sync::Arc};
use tokio::sync::oneshot::Receiver;

type DatabaseRef = Arc<Mutex<Database>>;

#[derive(Debug)]
pub enum Signal {
    Shutdown,
}

pub async fn service(database: DatabaseRef, _signals: Receiver<Signal>) -> Result<()> {
    let addr = ([127, 0, 0, 1], 8080).into();
    let make_service = make_service_fn(move |_| {
        let database = database.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                process_request(database.clone(), req)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_service);

    info!("Listening on http://{}", addr);

    // Run this server for... forever!
    match server.await {
        Err(err) => error!("server error: {:?}", err),
        Ok(_) => info!("terminated"),
    }

    Ok(())
}

/// Parse the path of the request and dispatch the request to the
/// correct submodule based on content type.
async fn process_request(database: DatabaseRef, req: Request<Body>) -> Result<Response<Body>> {
    let resource = match Resource::from_str(req.uri().path()) {
        Ok(resource) => resource,
        Err(err) => return Ok(err.into()),
    };
    let content_type = get_content_type(&req);
    match content_type {
        Ok("application/x-www-form-urlencoded") => {
            match form::process_request(database, resource, req).await {
                Ok(result) => Ok(result),
                Err(err) => Ok(err.into()),
            }
        }

        Ok("application/json") => match json::process_request(database, resource, req).await {
            Ok(result) => Ok(result),
            Err(err) => Ok(err.into()),
        },
        Ok(value) => {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(format!(
                    "Bad Content-Type {}: use either \
                     application/json or \
                     application/x-www-form-urlencoded",
                    value
                )));
            match response {
                Ok(result) => Ok(result),
                Err(_err) => todo!(),
            }
        }
        Err(_) => todo!(),
    }
}
