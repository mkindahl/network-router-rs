//! Resources (routes) for JSON.

use crate::{
    session::{DbRef, Rule},
    web::{handlers, with_db},
};
use warp::Filter;

/// List all available rules.
///
/// Returns a JSON object with the available rules.
pub(crate) fn list_rules(
    db: DbRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("rules")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::list_rules)
}

pub(crate) fn create_rule(
    db: DbRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("rules")
        .and(warp::post())
        .and(json_body())
        .and(with_db(db))
        .and_then(handlers::create_rule)
}

pub(crate) fn delete_rule(
    db: DbRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("rules" / usize)
        .and(warp::delete())
        .and(with_db(db))
        .and_then(handlers::delete_rule)
}

pub(crate) fn update_rule(
    db: DbRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("rules" / usize)
        .and(warp::put())
        .and(with_db(db))
        .and_then(handlers::update_rule)
}

fn json_body() -> impl Filter<Extract = (Rule,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
