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

extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate router;
extern crate yaml_rust;

use clap::{App, Arg};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use log::{debug, error, info};
use router::config::{Config, Protocol};
use router::strategy::{Mode, Strategy};
use router::tcp::TcpSession;
use router::udp::UdpSession;

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = App::new("Network Router")
        .version("0.1")
        .author("Mats Kindahl <mats.kindahl@gmail.com>")
        .about("Simple connection-based network router implemented in Rust using Tokio.")
        .arg(
            Arg::new("config_file")
                .short('f')
                .long("config-file")
                .value_name("FILE")
                .about("Read config from FILE")
                .takes_value(true),
        )
        .arg(
            Arg::new("config_string")
                .short('c')
                .long("config")
                .value_name("STRING")
                .about("Read config from STRING")
                .takes_value(true),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple(true)
                .about("Sets the level of verbosity"),
        )
        .get_matches();

    // Config string takes precedence, if given.
    let config = match matches.value_of("config_string") {
        Some(config_string) => {
            Config::read_from_string(&config_string).expect("Unable to read config string")
        }
        None => {
            let config_file = matches.value_of("config_file").unwrap_or("config.yaml");
            Config::read_from_file(&config_file).expect("unable to read config file")
        }
    };

    let mut sessions = FuturesUnordered::new();

    for section in config.sections {
        match section.protocol {
            Protocol::Udp(mode) => {
                let strategy = Strategy::new(mode, &section.destinations);
                for source in section.sources {
                    debug!("Spawning UDP session listening on {}", source);
                    sessions.push(tokio::spawn({
                        let strategy = strategy.clone();
                        async move { UdpSession::new(source, strategy).run().await }
                    }));
                }
            }

            Protocol::Tcp => {
                for source in section.sources {
                    let strategy = Strategy::new(Mode::RoundRobin, &section.destinations);
                    debug!("Spawning TCP session listening on {}", source);
                    sessions.push(tokio::spawn({
                        let strategy = strategy.clone();
                        async move { TcpSession::new(source, strategy).run().await }
                    }));
                }
            }
        }
    }

    while let Some(item) = sessions.next().await {
        match item {
            Ok(result) => info!("session exited {:?}", result),
            Err(err) => error!("error: {}", err),
        }
    }
}