//! HTTP server
//!
//! There are rules that decide what port to listen on and what ports
//! to use for the outgoing connections or packages.
//!
//! GET is used to read information:
//!
//! /            Show a summary
//! /rule        Show a summary of all rules
//! /rule/123    Show information about rule 123
//!
//! POST is used to update existing objects.
//!
//! PUT is used to create new objects.
//!
//! DELETE is used to delete objects.

use crate::storage::{Database, Rule};
use futures::lock::Mutex;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;

#[derive(Debug)]
pub enum Signal {
    Shutdown,
}

type DbRef = Arc<Mutex<Database>>;

fn make_row(rule: &Rule) -> String {
    format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        rule.mode,
        rule.source,
        rule.destinations
            .iter()
            .map(|addr| addr.to_string())
            .collect::<Vec<String>>()
            .join("<br/>")
    )
}

async fn make_table_rows(dbref: DbRef) -> String {
    dbref
        .lock()
        .await
        .rules
        .iter()
        .map(|(_id, rule)| make_row(rule))
        .collect::<Vec<String>>()
        .join("")
}

async fn process_request(
    database: DbRef,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from(format!(
            r#"
<html>
  <body>
    <table border="1"><tr><th>Mode</th><th>Source</th><th>Destinations</th></tr>{}</table>
  </body>
</html>
"#,
            make_table_rows(database).await
        )))),

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub async fn web_service(
    database: DbRef,
    signals: Receiver<Signal>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // This shutdown method is not particularly useful right now, but
    // keeping it here anyway since it will be more useful later.
    let graceful = server.with_graceful_shutdown(async move {
        match signals.await {
            Ok(Signal::Shutdown) => info!("shutting down web server"),
            Err(_) => error!("the sender dropped"),
        }
    });

    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
