//! Module that handles JSON requests.
//!
//! JSON structure contains the following fields:
//!
//!

mod handlers;
mod resources;

use crate::session::{Action, DbRef};
use std::{convert::Infallible, net::SocketAddr};
use tokio::sync::oneshot::Receiver;
use warp::{self, Filter};

fn with_db(db: DbRef) -> impl Filter<Extract = (DbRef,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn resources(
    db: DbRef,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    resources::list_rules(db.clone())
        .or(resources::update_rule(db.clone()))
        .or(resources::create_rule(db.clone()))
        .or(resources::delete_rule(db))
}

pub async fn service(db: DbRef, addr: SocketAddr, _signals: Receiver<Action>) {
    warp::serve(resources(db)).run(addr).await;
}
