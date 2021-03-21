//! Module that handles JSON requests.
//!
//! JSON structure contains the following fields:
//!
//!

mod handlers;
mod resources;

use crate::session::DbRef;
use std::convert::Infallible;
use warp::{self, Filter};

pub(crate) fn with_db(db: DbRef) -> impl Filter<Extract = (DbRef,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub(crate) fn resources(
    db: DbRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    resources::list_rules(db.clone()).or(resources::create_rule(db))
}
