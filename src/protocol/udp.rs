// Copyright 2019 Mats Kindahl
//
// Licensed under the Apache License, Version 2.0 (the "License"); you
// may not use this file except in compliance with the License.  You
// may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.  See the License for the specific language governing
// permissions and limitations under the License.

use crate::{
    protocol::Result,
    session::{strategy::Strategy, Rule},
};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct UdpSession {
    source: SocketAddr,
    strategy: Box<dyn Strategy + Send>,
}

/// An UDP session that will listen on one socket and send the packets
/// to one or more other sockets.
impl UdpSession {
    pub async fn new(rule: &Rule, strategy: Box<dyn Strategy + Send>) -> UdpSession {
        UdpSession {
            source: rule.source,
            strategy,
        }
    }

    /// Start the session.
    ///
    /// This will take ownership of the session and run it until a
    /// shutdown.
    pub async fn start(self) -> Result<()> {
        let UdpSession {
            source,
            mut strategy,
        } = self;

        let socket = UdpSocket::bind(&source).await?;

        info!("session started listening on {}", source);
        loop {
            let mut buf = [0; 1500];
            let bytes = socket.recv(&mut buf).await?;
            if bytes == 0 {
                break;
            }
            for addr in &strategy.destinations() {
                socket.send_to(&buf[0..bytes], &addr).await?;
            }
        }
        info!("session terminated");
        Ok(())
    }
}
