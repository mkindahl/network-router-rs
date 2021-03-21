//! Handlers for JSON requests

use crate::{session::Rule, web::DbRef};
use serde::Serialize;
use std::convert::Infallible;
use warp::{self, http::StatusCode};

#[derive(Serialize)]
struct CreateReply {
    rule_id: usize,
}

pub(crate) async fn list_rules(db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let handle = db.read().await;
    let rules = handle.rules.clone();
    Ok(warp::reply::json(&rules))
}

pub(crate) async fn create_rule(rule: Rule, db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let mut handle = db.write().await;
    let id = handle.add_rule(rule);
    // TODO() Create reply with the rule id.
    let json = warp::reply::json(&CreateReply { rule_id: id });
    Ok(warp::reply::with_status(json, StatusCode::CREATED))
}
