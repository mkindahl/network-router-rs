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
extern crate yaml_rust;

use std::net::SocketAddr;
use std::{fs, io};
use tokio::codec::BytesCodec;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::*;
use yaml_rust::YamlLoader;

/// Get configuration information from filename.
///
/// Configurations are written in YAML and each document (section of
/// the configuration, in YAML vocabulary) contains a protocol, a
/// source address to listen on, and a list of destination addresses
/// to forward to.
///
/// # Example
///
/// Here is a simple configuration that will listen on port 8080 and
/// forward to ports 8081 and 8082.
/// ```
/// ---
/// protocol: udp
/// source:
///   127.0.0.1:8080
/// destination:
///   127.0.0.1:8081
///   127.0.0.1:8082
/// ...
/// ```

fn get_config(filename: &str) -> io::Result<(UdpSocket, Vec<SocketAddr>)> {
    let yaml = {
        let config_text = fs::read_to_string(filename)?;
        YamlLoader::load_from_str(&config_text)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Configuration read problem"))?
    };

    // TODO(mkindahl): Make this code simpler and more generic.
    for part in yaml.iter() {
        if part["protocol"].as_str() == Some("udp") {
            let socket = if let Some(address) = part["source"].as_str() {
                let sockaddr = address.parse::<SocketAddr>().map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "Configuration read problem")
                })?;
                UdpSocket::bind(&sockaddr)?
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Configuration read problem",
                ));
            };

            let peers = if let Some(dests) = part["destination"].as_str() {
                dests
                    .split(" ")
                    .map(|addr| addr.parse::<SocketAddr>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "Configuration read problem")
                    })?
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Configuration read problem",
                ));
            };

            return Ok((socket, peers));
        }
    }
    return Err(io::Error::new(io::ErrorKind::Other, "No udp configuration"));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let (socket, peers) = get_config("config.yaml")?;
    debug!("listening on socket {:?}", socket);
    debug!("sending to peers {:?}", peers);
    let (mut writer, reader) = UdpFramed::new(socket, BytesCodec::new()).split();
    let forwarder = reader.for_each(move |(bytes, _from)| {
        let packet = bytes.freeze();
        for peer in peers.iter() {
            writer.start_send((packet.clone(), peer.clone()))?;
        }
        writer.poll_complete()?;
        Ok(())
    });

    tokio::run({
        forwarder
            .map_err(|err| error!("Error: {}", err))
            .map(|_| ())
    });
    Ok(())
}
