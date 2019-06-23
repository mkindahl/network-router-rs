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

use bytes::BytesMut;
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::*;

struct UdpSession {
    source: SocketAddr,
    peers: Vec<SocketAddr>,
}

impl UdpSession {
    fn new(source: &SocketAddr, destinations: &Vec<SocketAddr>) -> std::io::Result<UdpSession> {
        let mut peers: Vec<SocketAddr> = Vec::new();
        for dest in destinations {
            peers.push(*dest);
        }
        let session = UdpSession {
            source: source,
            peers: peers,
        };
        Ok(session)
    }
}

impl future::IntoFuture for UdpSession {
    type Future = future::Future<Item = Self::Item, Error = Self::Error>;
    type Item = ();
    type Error = ();

    fn into_future(&self) {
        let socket = UdpSocket::bind(&self.source)?;
        let (mut writer, reader) = UdpFramed::new(socket, BytesCodec::new()).split();
        let forward_packet = move |(bytes, _from): (BytesMut, SocketAddr)| {
            let packet = bytes.freeze();
            for peer in self.peers.iter() {
                writer.start_send((packet.clone(), peer.clone()))?;
            }
            writer.poll_complete()?;
            Ok(())
        };

        future::lazy(move || {
            reader
                .for_each(forward_packet)
                .map_err(|err| error!("error: {}", err))
                .map(|_| ())
        })
    }
}
