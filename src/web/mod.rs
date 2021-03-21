//! Web interface subsystem.
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
//! POST is used to add rules or routes.
//!
//! Since the standard only support POST and GET for forms, the
//! parameter `action` will be used to indicate the operation for
//! urlencoded data, but all operations are supported with JSON.
//!
//! * `/rules`: Used when adding a new rule. Will return a reference
//!   to the newly added rule.
//!
//! * `/rules/123`: Direct rule references will be used when modifying
//!   or deleting a rule. Note that modification of a rules does not
//!   affect existing routes in that rule.
//!
//! * `/rules/123/routes/1`: Direct reference to a specific route. It
//!   is only possible to delete (shutdown) a route, not modify an
//!   existing route.

mod error;
mod json;

use crate::session::{Action, Database};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot::Receiver, RwLock};

type DbRef = Arc<RwLock<Database>>;

pub async fn service(db: DbRef, addr: SocketAddr, _signals: Receiver<Action>) {
    warp::serve(json::resources(db)).run(addr).await;
}
