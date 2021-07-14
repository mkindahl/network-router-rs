// Copyright 2019-21 Mats Kindahl
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
use log::debug;
use router::{config::Config, session::Manager};
use std::{error::Error, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = App::new("Network Router")
        .version("0.2")
        .author("Mats Kindahl <mats.kindahl@gmail.com>")
        .help("Simple port-based network router implemented in Rust using Tokio.")
        .arg(
            Arg::with_name("config_file")
                .short("f")
                .long("config-file")
                .value_name("FILE")
                .help("Read configuration from FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config_string")
                .short("c")
                .long("config-string")
                .value_name("STRING")
                .help("Read configuration from STRING")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    // Config string takes precedence, if given.
    //
    // Consider if we should allow the config string to just override
    // the configuration in the config file, if a config file is
    // given.
    let config = match matches.value_of("config_string") {
        Some(config_string) => Config::from_str(&config_string)?,
        None => {
            let config_file = matches.value_of("config_file").unwrap_or("config.json");
            debug!("Reading configuration from file '{}'", config_file);
            Config::from_file(&config_file)?
        }
    };

    let mut manager = Manager::new();
    for rule in config.rules {
        manager.add_rule(rule).await;
    }

    manager.start().await;
    Ok(())
}
