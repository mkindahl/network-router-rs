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
use std::env;
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{TcpListener, TcpStream, UdpFramed, UdpSocket};
use tokio::prelude::*;

// Spawn a new UDP session
fn spawn_udp_session(source: SocketAddr, peers: Vec<SocketAddr>) {
    let socket = UdpSocket::bind(&source).expect("unable to bind UDP socket");
    let (mut writer, reader) = UdpFramed::new(socket, BytesCodec::new()).split();
    tokio::spawn({
        reader
            .for_each(move |(bytes, _from): (BytesMut, SocketAddr)| {
                let packet = bytes.freeze();
                for peer in peers.iter() {
                    writer.start_send((packet.clone(), peer.clone()))?;
                }
                writer.poll_complete()?;
                Ok(())
            })
            .map_err(|err| error!("error: {}", err))
            .map(|_| ())
    });
}

/// Spawn a new TCP listener.
fn spawn_tcp_listener(source: SocketAddr, destination: SocketAddr) {
    let socket = TcpListener::bind(&source).expect("unable to bind TCP listener");

    tokio::spawn({
        socket
            .incoming()
            .for_each(move |client| {
                spawn_tcp_session(client, destination.clone());
                Ok(())
            })
            .map_err(|err| {
                error!("error: {}", err);
            })
            .map(|_| ())
    });
}

/// Spawn a new TCP session.
fn spawn_tcp_session(client: TcpStream, destination: SocketAddr) {
    tokio::spawn({
        TcpStream::connect(&destination)
            .and_then(|server| {
                let (client_reader, client_writer) = client.split();
                let (server_reader, server_writer) = server.split();
                let client_to_server = tokio::io::copy(client_reader, server_writer)
                    .and_then(|(_, _, mut server_writer)| {
                        info!("Shutting down connection to server");
                        server_writer.shutdown()
                    })
                    .map(|_| ());
                let server_to_client = tokio::io::copy(server_reader, client_writer)
                    .and_then(|(_, _, mut client_writer)| {
                        info!("Shutting down connection to client");
                        client_writer.shutdown()
                    })
                    .map(|_| ());
                client_to_server.join(server_to_client)
            })
            .map(move |_| ())
            .map_err(|err| {
                error!("error: {}", err);
            })
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config_file = env::args().nth(1).unwrap_or("config.yaml".to_string());
    let config = Config::read_from_file(&config_file)?;

    tokio::run(future::lazy(|| {
        for section in config.sections {
            match section.protocol {
                Protocol::Udp => {
                    for source in section.sources {
                        spawn_udp_session(source.clone(), section.destinations.clone());
                    }
                }

                Protocol::Tcp => {
                    for source in section.sources {
                        spawn_tcp_listener(source.clone(), section.destinations[0].clone());
                    }
                }
            }
        }
        Ok(())
    }));
    Ok(())
}
