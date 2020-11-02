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

use clap::{App, Arg};
use futures::lock::Mutex;
use futures::{stream::FuturesUnordered, StreamExt};
use log::{debug, error, info};
use router::{
    config::Config,
    protocol::{tcp::TcpSession, udp::UdpSession},
    storage::{Database, Protocol},
    strategy::*,
    web,
};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = App::new("Network Router")
        .version("0.1")
        .author("Mats Kindahl <mats.kindahl@gmail.com>")
        .about("Simple port-based network router implemented in Rust using Tokio.")
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
            Config::from_str(&config_string).expect("Unable to read config string")
        }
        None => {
            let config_file = matches.value_of("config_file").unwrap_or("config.yaml");
            Config::from_file(&config_file).expect("unable to read config file")
        }
    };

    let mut sessions = FuturesUnordered::new();

    let database = Arc::new(Mutex::new(Database::new()));

    let (sender, receiver) = tokio::sync::oneshot::channel::<web::Signal>();
    let web_service = tokio::spawn({
        let database = database.clone();
        web::service(database, receiver)
    });

    for rule in config.rules {
        match rule.protocol {
            Protocol::Udp => {
                debug!("UDP session listening on {}", rule.source);
                sessions.push(tokio::spawn({
                    let strategy = StrategyFactory::make(rule.mode, &rule.destinations);
                    let database = database.clone();
                    UdpSession::new(database, rule, strategy).await.start()
                }));
            }

            Protocol::Tcp => {
                debug!("TCP session listening on {}", rule.source);
                sessions.push(tokio::spawn({
                    let strategy = StrategyFactory::make(rule.mode, &rule.destinations);
                    let database = database.clone();
                    TcpSession::new(database, rule, strategy).await.start()
                }));
            }
        }
    }

    while let Some(item) = sessions.next().await {
        match item {
            Ok(result) => info!("session exited {:?}", result),
            Err(err) => error!("error: {}", err),
        }
    }

    if let Err(err) = sender.send(web::Signal::Shutdown) {
        eprintln!("shutdown error: {:?}", err);
    }

    if let Err(e) = web_service.await {
        eprintln!("server error: {}", e);
    }
}
