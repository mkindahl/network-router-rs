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

use crate::strategy::Strategy;
use log::debug;
use std::error;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct UdpSession {
    source: SocketAddr,
    strategy: Strategy,
}

/// An UDP session that will listen on one socket and send the packets
/// to one or more other sockets.
impl UdpSession {
    /// Create a new session
    pub fn new(source: SocketAddr, strategy: Strategy) -> UdpSession {
        UdpSession { source, strategy }
    }

    /// Run a session.
    ///
    /// This will take ownership of the session and run it until a
    /// shutdown.
    pub async fn run(self) -> Result<(), Box<dyn error::Error + Send>> {
        let UdpSession {
            source,
            mut strategy,
        } = self;

        debug!("Starting UDP session");
        let mut socket = match UdpSocket::bind(&source).await {
            Ok(socket) => socket,
            Err(err) => return Err(Box::new(err)),
        };
        debug!("Bound socket {}", source);
        loop {
            let mut buf = [0; 1500];
            let bytes = match socket.recv(&mut buf).await {
                Ok(bytes) => bytes,
                Err(err) => return Err(Box::new(err)),
            };
            debug!("Receiving {} bytes", bytes);
            if bytes == 0 {
                break;
            }
            for addr in &strategy.destinations() {
                debug!("Sending {} bytes to address {}", bytes, addr);
                if let Err(err) = socket.send_to(&buf[0..bytes], &addr).await {
                    return Err(Box::new(err));
                }
            }
        }
        debug!("Terminating UDP session");
        Ok(())
    }
}
