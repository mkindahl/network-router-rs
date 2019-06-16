//! Configurations are written in YAML and each document (section of
//! the configuration, in YAML vocabulary) contains a protocol, a
//! source address to listen on, and a list of destination addresses
//! to forward to.
//!
//! # Example
//!
//! Here is a simple configuration that will listen on port 8080 and
//! forward to ports 8081 and 8082.
//! ```
//! ---
//! protocol: udp
//! source:
//!   127.0.0.1:8080
//! destination:
//!   127.0.0.1:8081
//!   127.0.0.1:8082
//! ...
//! ```

use std::net::SocketAddr;
use yaml_rust::{Yaml, YamlLoader};

/// Protocol description
pub enum Protocol {
    Udp {
        source: SocketAddr,
        destinations: Vec<SocketAddr>,
    },
    Tcp {
        source: SocketAddr,
        destinations: Vec<SocketAddr>,
    },
}

pub struct Config {
    pub sections: Vec<Protocol>,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    YamlError(yaml_rust::scanner::ScanError),
    ConfigFileError,
    AddressParseError,
    MissingDestinationAddress,
    MissingSourceAddress,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO make better error
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::IoError(err) => err.description(),
            Error::YamlError(err) => err.description(),
            Error::ConfigFileError => "configuration file error",
            Error::AddressParseError => "malformed address",
            Error::MissingDestinationAddress => "missing destination address",
            Error::MissingSourceAddress => "missing source address",
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl std::convert::From<yaml_rust::scanner::ScanError> for Error {
    fn from(error: yaml_rust::scanner::ScanError) -> Self {
        Error::YamlError(error)
    }
}

impl std::convert::From<std::net::AddrParseError> for Error {
    fn from(_: std::net::AddrParseError) -> Self {
        Error::AddressParseError
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn parse_address(yaml: &Yaml) -> Result<SocketAddr> {
    match yaml {
        Yaml::String(addr) => Ok(addr.parse::<SocketAddr>()?),
        _ => Err(Error::AddressParseError),
    }
}

fn make_protocol(
    prot: &Yaml,
    source: SocketAddr,
    destinations: Vec<SocketAddr>,
) -> Result<Protocol> {
    match prot {
        Yaml::String(ref proto) if proto == "udp" => Ok(Protocol::Udp {
            source,
            destinations,
        }),
        Yaml::String(ref proto) if proto == "tcp" => Ok(Protocol::Tcp {
            source,
            destinations,
        }),
        _ => Err(Error::AddressParseError),
    }
}

impl Config {
    pub fn new() -> Config {
        Config {
            sections: Vec::new(),
        }
    }

    pub fn add(&mut self, protocol: Protocol) {
        self.sections.push(protocol)
    }

    pub fn load_from_file(filename: &str) -> Result<Config> {
        let yaml = {
            let config_text = std::fs::read_to_string(filename)?;
            YamlLoader::load_from_str(&config_text)?
        };

        let mut config = Config::new();
        for part in yaml.iter() {
            let source = parse_address(&part["source"])?;
            debug!("parsing destinations {:?}", &part["destination"]);
            let peers = match &part["destination"] {
                Yaml::Array(dests) => dests
                    .iter()
                    .map(|addr| parse_address(addr))
                    .collect::<Result<Vec<_>>>()?,
                _ => return Err(Error::MissingDestinationAddress),
            };
            config.add(make_protocol(&part["protocol"], source, peers)?);
        }
        Ok(config)
    }
}
