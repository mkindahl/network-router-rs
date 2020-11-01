//! TCP session module.
//!
//! A lot of the code is copied from the `proxy.rs` example in the
//! Tokio examples directory.

use crate::strategy::Strategy;
use futures::{future, FutureExt};
use std::error;
use std::net::SocketAddr;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct TcpSession {
    source: SocketAddr,
    strategy: Box<dyn Strategy + Send>,
}

/// A TCP session.
///
/// The TCP session will listen for connections on the provided port
/// and send to the provided destination.
impl TcpSession {
    pub fn new(source: SocketAddr, strategy: Box<dyn Strategy + Send>) -> TcpSession {
        TcpSession { source, strategy }
    }

    pub async fn run(self) -> Result<(), Box<dyn error::Error + Send>> {
        let TcpSession {
            source,
            mut strategy,
        } = self;
        let mut listener = match TcpListener::bind(source).await {
            Ok(listener) => listener,
            Err(err) => return Err(Box::new(err)),
        };

        info!("session started listening for connections");
        while let Ok((client, client_addr)) = listener.accept().await {
            info!("accepting connection from {}", client_addr);
            let destinations = strategy.destinations();
            assert!(destinations.len() == 1);
            let transfer = transfer(client, destinations[0]).map(|result| {
                if let Err(err) = result {
                    debug!("Failed to transfer; error={}", err);
                }
            });
            tokio::spawn(transfer);
        }
        Ok(())
    }
}

/// Set up a bidirectional connection.
///
/// This is copied from the `proxy.rs` example in the Tokio examples
/// directory.
///
/// Intention is to refactor this to allow some basic packet
/// inspection to handle SSL connections.
async fn transfer(
    mut inbound: TcpStream,
    proxy_addr: SocketAddr,
) -> Result<(), Box<dyn error::Error>> {
    info!("connecting to {}", proxy_addr);
    let mut outbound = TcpStream::connect(proxy_addr).await?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        info!("shutting down connection");
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        info!("shutting down connection");
        wi.shutdown().await
    };

    future::try_join(client_to_server, server_to_client).await?;

    info!("session terminated");
    Ok(())
}
