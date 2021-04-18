pub mod rules;
pub mod strategy;

use crate::{protocol, protocol::udp::UdpSession, rest, session::strategy::StrategyFactory};
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
pub use rules::{Database, Mode, Protocol, Route, Rule};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
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
    addr: SocketAddr,
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
    pub fn new() -> Manager {
        Manager {
            sender: None,
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2357),
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
        let (sender, receiver) = tokio::sync::oneshot::channel::<Action>();
        let service = tokio::spawn({
            let database = self.database.clone();
            rest::service(database, self.addr, receiver)
        });
        self.sender = Some(sender);

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

        if let Err(e) = service.await {
            eprintln!("server error: {}", e);
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
