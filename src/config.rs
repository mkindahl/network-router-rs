//! Configuration parser.
//!
//! Configurations are written in YAML and each document (section of
//! the configuration, in YAML vocabulary) contains a forwarding
//! description.
//!
//! # Sections
//!
//! Each section contains:
//!
//! - A protocol, which can be either "tcp" or "udp"
//!
//! - A mode, which can be either "broadcast" or
//!   "round-robin". Defaults to "round-robin" for TCP and to
//!   "broadcast" for UDP.
//!
//!   Note that if there is a single destination address, then the
//!   mode is irrelevant since the behaviour is identical for both
//!   modes.
//!
//! - One or more source addresses to listen on.
//!
//! - One or more destination addresses to forward to.
//!
//! ## Broadcast Mode
//!
//! In broadcast mode, each packet received on a source address is
//! distributed to each destination address. Broadcast only makes
//! sense for UDP so an error will be given if the protocol is TCP and
//! there is more than one destination address.
//!
//! ## Round-Robin Mode
//!
//! In TCP round-robin mode, a connection on a source addresses will
//! be established to one of the destination addresses in a
//! round-robin manner.
//!
//! For UDP, the packets are sent to the destination ports in a
//! round-robin fashion.
//!
//!# Example
//!
//! Here is a simple configuration that will listen on port 8080 and
//! forward to ports 8081 and 8082.
//! ```yaml
//! ---
//! protocol: udp
//! source:
//!   - "127.0.0.1:8080"
//! destination:
//!   - "127.0.0.1:8081"
//!   - "127.0.0.1:8082"
//!...
//! ```

use crate::strategy::{self, Mode};
use std::fs;
use std::net::SocketAddr;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    YamlError(yaml_rust::scanner::ScanError),
    ConfigError(String),
    SyntaxError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "I/O error: {}", err),
            Error::YamlError(err) => write!(f, "YAML error: {}", err),
            Error::ConfigError(ref txt) => write!(f, "Config error: {}", txt),
            Error::SyntaxError(ref txt) => write!(f, "Syntax error: {}", txt),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::IoError(_) => "I/O error",
            Error::YamlError(_) => "YAML error",
            Error::ConfigError(_) => "config error",
            Error::SyntaxError(_) => "syntax error",
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl std::convert::From<strategy::Error> for Error {
    fn from(err: strategy::Error) -> Self {
        Error::SyntaxError(format!("{}", err))
    }
}

impl std::convert::From<yaml_rust::scanner::ScanError> for Error {
    fn from(error: yaml_rust::scanner::ScanError) -> Self {
        Error::YamlError(error)
    }
}

impl std::convert::From<std::net::AddrParseError> for Error {
    fn from(_: std::net::AddrParseError) -> Self {
        Error::ConfigError("not an address".to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Protocol
#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Udp(Mode),
    Tcp,
}

/// A section in the configuration file.
#[derive(Debug)]
pub struct Section {
    pub protocol: Protocol,
    pub sources: Vec<SocketAddr>,
    pub destinations: Vec<SocketAddr>,
}

/// Parse a YAML node into a socket address.
fn parse_address(yaml: &Yaml) -> Result<SocketAddr> {
    match yaml {
        Yaml::String(addr) => Ok(addr.parse::<SocketAddr>()?),
        _ => Err(Error::ConfigError(format!(
            "{:?} is not a valid address",
            yaml
        ))),
    }
}

/// Parse YAML item into a list of socket addresses.
fn parse_address_list(yaml: &Yaml) -> Result<Vec<SocketAddr>> {
    match yaml {
        Yaml::Array(ref dests) => dests
            .iter()
            .map(|addr| parse_address(addr))
            .collect::<Result<Vec<_>>>(),
        _ => Err(Error::ConfigError(format!(
            "Malformed configuration: {:?}",
            yaml
        ))),
    }
}

impl Section {
    pub fn from_yaml(
        protocol_field: &Yaml,
        mode_field: &Yaml,
        sources: &Yaml,
        destinations: &Yaml,
    ) -> Result<Section> {
        let mode = match mode_field {
            Yaml::String(ref text) => text.parse()?,
            other => {
                return Err(Error::ConfigError(format!(
                    "cannot be parsed as a mode: {:?}",
                    other
                )));
            }
        };
        let protocol = match protocol_field {
            Yaml::String(ref proto) if proto == "udp" => Protocol::Udp(mode),
            Yaml::String(ref proto) if proto == "tcp" => Protocol::Tcp,
            Yaml::String(ref txt) => {
                return Err(Error::ConfigError(format!(
                    "'{}' is not a valid protocol",
                    txt
                )));
            }
            other => {
                return Err(Error::ConfigError(format!(
                    "cannot be parsed as a protocol: {:?}",
                    other
                )));
            }
        };

        let section = Section {
            protocol,
            sources: parse_address_list(sources)?,
            destinations: parse_address_list(destinations)?,
        };

        debug!("Found {:?}", section);

        Ok(section)
    }
}

/// Configuration with sections.
pub struct Config {
    pub sections: Vec<Section>,
}

impl Config {
    /// Create a new empty configuration.
    pub fn new() -> Config {
        Config {
            sections: Vec::new(),
        }
    }

    /// Add a section to the configuration.
    pub fn add(&mut self, section: Section) {
        self.sections.push(section)
    }

    fn read_from_vec(yaml: Vec<Yaml>) -> Result<Config> {
        let mut config = Config::new();
        for part in yaml.iter() {
            config.add(Section::from_yaml(
                &part["protocol"],
                &part["mode"],
                &part["sources"],
                &part["destinations"],
            )?);
        }
        Ok(config)
    }

    pub fn read_from_string(text: &str) -> Result<Config> {
        Self::read_from_vec(YamlLoader::load_from_str(text)?)
    }

    /// Read a YAML configuration from a file name.
    pub fn read_from_file(filename: &str) -> Result<Config> {
        info!("Loading configuration from {}", filename);
        Self::read_from_string(&fs::read_to_string(filename)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Udp(mode) => write!(f, "UDP mode: {}", mode),
            Protocol::Tcp => write!(f, "TCP"),
        }
    }
}
