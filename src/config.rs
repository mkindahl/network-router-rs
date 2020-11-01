//! Configurations are written in JSON and consists of one section for
//! each system:
//!
//! - Web interface
//! - Forwarding rules
//!
//! # Web interface
//!
//! Web interface configuration is under "web" key and has the
//! following fields:
//!
//! - **port** is the port to listen on. If it is "*", then it means
//! pick a random port to listen on.
//!
//! - **address** is a full address to listen on. This can be used for
//! machines that have several network interfaces.
//!
//! # Forwarding rules
//!
//! Each rule section can contain four different attributes:
//!
//! - **protocol** is the protocol that the section should use. It can be
//!   either `Udp` or `Tcp` (it is case-sensitive).
//! - **mode** can be either `Broadcast` or `RoundRobin` and the default
//!   is `Broadcast` for UDP and `RoundRobin` for TCP.
//!  
//!   - In broadcast mode, each packet will be sent to all destinations,
//!     which only make sense for UDP.
//!
//!   - In round-robin mode, each packet will be sent to or connection
//!     established with one target at a time in a round-robin fashion.
//!
//! - **source** is a source addresses that the router should
//!   listen on.
//!  
//! - **destinations** is a list of destination addresses that the router
//!   should send packets or establish connections with.
//!
//! # Example
//!
//! Here is a simple configuration that will broadcast UDP traffic
//! from port 9080 and forward to ports 9081 and 9082 and forward TCP
//! connections in a round-robin fashion from port 9090 to 9091 and
//! 9092.
//!
//! ```json
//! {
//!     "web": {
//!         "port": "8080",
//!     },
//!     "rules": [
//!         {
//!             "protocol":"Udp",
//!             "mode":"broadcast",
//!             "source": "127.0.0.1:9080",
//!             "destinations": [
//!                 "127.0.0.1:9081",
//!                 "127.0.0.1:9082"
//!             ]
//!         },
//!         {
//!             "protocol":"tcp",
//!             "mode":"round-robin",
//!             "source": "127.0.0.1:9090",
//!             "destinations": [
//!                 "127.0.0.1:9081",
//!                 "127.0.0.1:9082"
//!             ]
//!         },
//!     ]
//! }

use crate::session::{strategy, Rule};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr};

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Web {
    Port(Option<u16>),
    Address(SocketAddr),
}

/// Configuration with rules.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<Web>,
    pub rules: Vec<Rule>,
}

#[derive(PartialEq, Debug)]
pub enum Error {
    IoError(String),
    JsonError(String),
    ConfigError(String),
    SyntaxError(String),
}

impl Config {
    pub fn from_json(json: &str) -> Result<Config> {
        let config: Config = serde_json::from_str(json)?;
        Ok(config)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|err| Error::JsonError(format!("JSON Error: {}", err)))
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "I/O error: {}", err),
            Error::JsonError(err) => write!(f, "JSON error: {}", err),
            Error::ConfigError(ref txt) => write!(f, "Config error: {}", txt),
            Error::SyntaxError(ref txt) => write!(f, "Syntax error: {}", txt),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::IoError(_) => "I/O error",
            Error::JsonError(_) => "JSON error",
            Error::ConfigError(_) => "config error",
            Error::SyntaxError(_) => "syntax error",
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(format!("{}", error))
    }
}

impl std::convert::From<strategy::Error> for Error {
    fn from(err: strategy::Error) -> Self {
        Error::SyntaxError(format!("{}", err))
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(format!("{}", error))
    }
}

impl std::convert::From<std::net::AddrParseError> for Error {
    fn from(_: std::net::AddrParseError) -> Self {
        Error::ConfigError("not an address".to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl Rule {
    pub fn from_json(data: &str) -> Result<Rule> {
        let rule: Rule = serde_json::from_str(data)?;
        Ok(rule)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|err| Error::JsonError(format!("JSON Error: {}", err)))
    }
}

impl Config {
    /// Create a new empty configuration.
    pub fn new() -> Config {
        Config {
            web: None,
            rules: Vec::new(),
        }
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.web = Some(Web::Port(Some(port)));
        self
    }

    /// Add a rule to the configuration.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule)
    }

    /// Read a JSON configuration from a file name.
    pub fn from_file(filename: &str) -> Result<Config> {
        info!("Loading configuration using path '{}'", filename);
        let contents = fs::read_to_string(filename)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for Rule {
    type Err = Error;
    fn from_str(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|err| Error::JsonError(format!("{}", err)))
    }
}

impl std::str::FromStr for Config {
    type Err = Error;
    fn from_str(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|err| Error::JsonError(format!("{}", err)))
    }
}

impl std::str::FromStr for Web {
    type Err = Error;
    fn from_str(text: &str) -> Result<Self> {
        if let Ok(addr) = text.parse::<SocketAddr>() {
            Ok(Web::Address(addr))
        } else if let Ok(port) = text.parse::<u16>() {
            Ok(Web::Port(Some(port)))
        } else if text == "*" {
            Ok(Web::Port(None))
        } else {
            Err(Error::SyntaxError(format!(
                "'{}' is neither a port description nor a socket address",
                text
            )))
        }
    }
}

impl std::fmt::Display for Web {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Web::Port(Some(port)) => write!(f, "localhost:{}", port),
            Web::Port(None) => write!(f, "localhost:*"),
            Web::Address(addr) => write!(f, "{}", addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Mode, Protocol};

    #[test]
    fn test_rule_parse() {
        let rule: Result<Rule> =
            r#"{"protocol": "udp", "mode": "broadcast", "source": "127.0.0.1:8080", "destinations": []}"#
                .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:8080".parse().unwrap(),
                destinations: vec![]
            })
        );

        let rule: Result<Rule> = r#"{"protocol":"udp",
                "mode":"broadcast", "source": "127.0.0.1:9080",
                "destinations": []}"#
            .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec![]
            })
        );

        let rule: Result<Rule> = r#"{"protocol":"udp",
                "mode":"broadcast", "source": "127.0.0.1:9080",
                "destinations": ["127.0.0.1:9081", "127.0.0.1:9082"]}"#
            .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec![
                    "127.0.0.1:9081".parse().unwrap(),
                    "127.0.0.1:9082".parse().unwrap()
                ]
            })
        );

        let rule: Result<Rule> = r#"{"protocol":"udp",
                "mode":"broadcast", "source": "127.0.0.1:9080",
                "destinations": ["127.0.0.1:9081"]}"#
            .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec!["127.0.0.1:9081".parse().unwrap()]
            })
        );
    }

    #[test]
    fn test_config_parse() {
        let config: Result<Config> = r#"
{
    "web": {"port": 1111},
    "rules": [
	{
	    "protocol":"udp",
	    "mode":"broadcast",
	    "source": "127.0.0.1:9080",
	    "destinations": ["127.0.0.1:9081", "127.0.0.1:9082"]
	}
    ]
}
"#
        .parse();
        assert_eq!(
            config,
            Ok(Config {
                web: Some(Web::Port(Some(1111))),
                rules: vec![Rule {
                    protocol: Protocol::Udp,
                    mode: Mode::Broadcast,
                    source: "127.0.0.1:9080".parse().unwrap(),
                    destinations: vec![
                        "127.0.0.1:9081".parse().unwrap(),
                        "127.0.0.1:9082".parse().unwrap()
                    ]
                }]
            })
        );
    }
    #[test]
    fn test_config_parse_no_web() {
        let config: Result<Config> = r#"
{
    "rules": [
	{
	    "protocol":"udp",
	    "mode":"broadcast",
	    "source": "127.0.0.1:9080",
	    "destinations": ["127.0.0.1:9081", "127.0.0.1:9082"]
	}
    ]
}
"#
        .parse();
        assert_eq!(
            config,
            Ok(Config {
                web: None,
                rules: vec![Rule {
                    protocol: Protocol::Udp,
                    mode: Mode::Broadcast,
                    source: "127.0.0.1:9080".parse().unwrap(),
                    destinations: vec![
                        "127.0.0.1:9081".parse().unwrap(),
                        "127.0.0.1:9082".parse().unwrap()
                    ]
                }]
            })
        );
    }

    #[test]
    fn test_config_serialize_no_web() {
        let config = Config {
            web: None,
            rules: vec![Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec![
                    "127.0.0.1:9081".parse().unwrap(),
                    "127.0.0.1:9082".parse().unwrap(),
                ],
            }],
        };
        let result = r#"{"rules":[{"protocol":"udp","mode":"broadcast","source":"127.0.0.1:9080","destinations":["127.0.0.1:9081","127.0.0.1:9082"]}]}"#;
        assert_eq!(serde_json::to_string(&config).unwrap(), result.to_string());
    }

    #[test]
    fn test_web_parse() {
        assert_eq!(
            Web::Port(Some(4711)).to_string(),
            "localhost:4711".to_string()
        );
        assert_eq!(Web::Port(None).to_string(), "localhost:*".to_string());
    }
}
