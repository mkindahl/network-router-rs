pub mod rules;
pub mod strategy;

use crate::{
    config::Config, protocol, protocol::udp::UdpSession, session::strategy::StrategyFactory, web,
};
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
pub use rules::{Database, Mode, Protocol, Route, Rule};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    sync::{oneshot::Sender, RwLock},
    task::JoinHandle,
};

pub type DbRef = Arc<RwLock<Database>>;

pub enum Error {
    ShutdownFailed,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Action {
    Shutdown,
}

/// Sessions listen on sockets and process packets arriving over the
/// socket.
#[async_trait]
pub trait Session {
    /// Start executing the session. This will unpack the session and
    /// run it to completion.
    async fn run(self) -> Result<()>;
}

/// Session manager that handle the addition and removal of sessions
/// as well as answers requests for information about sessions.
pub struct Manager {
    sender: Option<Sender<Action>>,
    /// HTTP listen address for both JSON and HTML requests.
    addr: Option<SocketAddr>,
    database: DbRef,
    sessions: FuturesUnordered<JoinHandle<protocol::Result<()>>>,
}

impl Manager {
    pub async fn shutdown(&mut self) -> Result<()> {
        match self.sender.take() {
            Some(sender) => {
                sender
                    .send(Action::Shutdown)
                    .map_err(|_| Error::ShutdownFailed)?;
            }
            None => {
                error!("Router already shut down");
            }
        }
        Ok(())
    }

    /// Create a new manager but do not start it.
    pub fn new(config: &Config) -> Manager {
        Manager {
            sender: None,
            addr: config.http.map(|v| v.into()),
            database: Arc::new(RwLock::new(Database::new())),
            sessions: FuturesUnordered::new(),
        }
    }

    /// Adding a new rule to the session manager will create a session
    /// for the rule and add it to the set of tasks running as well as
    /// updating the database with all rules.
    pub async fn add_rule(&mut self, rule: Rule) {
        let session = tokio::spawn({
            let strategy = StrategyFactory::make(&rule);
            UdpSession::new(&rule, strategy).await.start()
        });
        self.sessions.push(session);
        self.database.write().await.create_rule(rule);
    }

    /// Start the manager by starting all tasks.
    pub async fn start(&mut self) {
        // Spawn HTTP API thread, if available.
        if let Some(addr) = self.addr {
            let (sender, receiver) = tokio::sync::oneshot::channel::<Action>();
            let http_service = tokio::spawn({
                let database = self.database.clone();
                web::service(database, addr, receiver)
            });
            self.sender = Some(sender);
        }

        while let Some(item) = self.sessions.next().await {
            match item {
                Ok(result) => info!("session exited {:?}", result),
                Err(err) => error!("error: {}", err),
            }
        }

        match self.sender.take() {
            Some(sender) => {
                if let Err(err) = sender.send(Action::Shutdown) {
                    eprintln!("shutdown error: {:?}", err);
                }
            }
            None => {
                error!("Router already shut down");
            }
        }
    }
}
