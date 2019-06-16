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

#[macro_use]
extern crate log;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate router;
extern crate yaml_rust;

use bytes::BytesMut;
use router::config::{Config, Protocol};
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::load_from_file("config.yaml")?;

    tokio::run(future::lazy(|| {
        for section in config.sections {
            match section {
                Protocol::Udp {
                    ref source,
                    ref destinations,
                } => {
                    let socket = UdpSocket::bind(source).unwrap();
                    let (mut writer, reader) = UdpFramed::new(socket, BytesCodec::new()).split();
                    let peers = destinations.clone();
                    let forward_packet = move |(bytes, _from): (BytesMut, SocketAddr)| {
                        let packet = bytes.freeze();
                        for destination in peers.iter() {
                            writer.start_send((packet.clone(), destination.clone()))?;
                        }
                        writer.poll_complete()?;
                        Ok(())
                    };

                    tokio::spawn(future::lazy(move || {
                        reader
                            .for_each(forward_packet)
                            .map_err(|err| error!("error: {}", err))
                            .map(|_| ())
                    }));
                }
                _ => (),
            }
        }
        Ok(())
    }));
    Ok(())
}
