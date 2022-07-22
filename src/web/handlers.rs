use crate::{session::Rule, web::DbRef};
use askama::Template;
use serde::Serialize;
use std::convert::Infallible;
use warp::{self, http::StatusCode};

#[derive(Template)]
#[template(path = "list_rules.html")]
struct AllRulesTemplate<'a> {
    rules: Vec<&'a Rule>,
}

#[derive(Serialize)]
struct CreateReply {
    rule_id: usize,
}

pub(crate) async fn list_rules(accept: String, db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let handle = db.read().await;
    let rules: Vec<_> = handle.rules.iter().filter_map(|x| x.as_ref()).collect();
    for fmt in accept.split(',').map(|s| s.trim()) {
        match fmt {
            "application/json" => return Ok(warp::reply::json(&rules)),
            "text/html" => {
                let body = AllRulesTemplate { rules };
                return Ok(warp::reply::html(body.render()));
            }
        }
    }
    Err()
}

pub(crate) async fn create_rule_json(
    db: DbRef,
    rule: Rule,
) -> Result<impl warp::Reply, Infallible> {
    let mut handle = db.write().await;
    let id = handle.create_rule(rule);
    let reply = CreateReply { rule_id: id };
    Ok(warp::reply::with_status(
        warp::reply::json(&reply),
        StatusCode::CREATED,
    ))
}

pub(crate) async fn create_rule_form(
    db: DbRef,
    rule: Rule,
) -> Result<impl warp::Reply, Infallible> {
    let mut handle = db.write().await;
    let id = handle.create_rule(rule);
    let rules: Vec<_> = handle.rules.iter().filter_map(|x| x.as_ref()).collect();
    let body = AllRulesTemplate { rules };
    Ok(warp::reply::with_status(
        warp::reply::html(body.render().expect("cannot render template")),
        StatusCode::CREATED,
    ))
}

pub(crate) async fn delete_rule(
    rule_id: usize,
    accept: String,
    db: DbRef,
) -> Result<impl warp::Reply, Infallible> {
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
