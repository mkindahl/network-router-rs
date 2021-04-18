//! Handlers for JSON requests

use crate::{rest::DbRef, session::Rule};
use serde::Serialize;
use std::convert::Infallible;
use warp::{self, http::StatusCode};

#[derive(Serialize)]
struct CreateReply {
    rule_id: usize,
}

pub(crate) async fn list_rules(db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let handle = db.read().await;
    let rules: Vec<&Rule> = handle.rules.iter().filter_map(|x| x.as_ref()).collect();
    Ok(warp::reply::json(&rules))
}

pub(crate) async fn create_rule(rule: Rule, db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let mut handle = db.write().await;
    let id = handle.create_rule(rule);
    let json = warp::reply::json(&CreateReply { rule_id: id });
    Ok(warp::reply::with_status(json, StatusCode::CREATED))
}

pub(crate) async fn delete_rule(rule_id: usize, db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let mut handle = db.write().await;
    match handle.drop_rule(rule_id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Ok(StatusCode::NOT_FOUND),
    }
}

pub(crate) async fn update_rule(
    rule_id: usize,
    rule: Rule,
    db: DbRef,
) -> Result<impl warp::Reply, Infallible> {
    let mut handle = db.write().await;
    match handle.update_rule(rule_id, rule) {
        Some(_) => Ok(StatusCode::OK),
        None => Ok(StatusCode::NOT_FOUND),
    }
}
