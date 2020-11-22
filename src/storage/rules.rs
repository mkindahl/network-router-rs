//! Rules module.
//!
//! Each rule contains:
//!
//! - A protocol, which can be either "Tcp" or "Udp"
//!
//! - A mode, which can be either "Broadcast" or
//!   "RoundRobin". Defaults to "RoundRobin" for TCP and to
//!   "Broadcast" for UDP.
//!
//!   Note that if there is a single destination address, then the
//!   mode is irrelevant since the behaviour is identical for both
//!   modes.
//!
//! - One or more source addresses to listen on. If more than one
//!   source address is given, this is the same as creating several
//!   separate rules with a single source address.
//!
//! - One or more destination addresses to forward to.
//!
//! # Broadcast Mode
//!
//! In broadcast mode, each packet received on a source address is
//! distributed to each destination address. Broadcast only makes
//! sense for UDP so an error will be given if the protocol is TCP and
//! there is more than one destination address.
//!
//! # Round-Robin Mode
//!
//! In TCP round-robin mode, a connection on a source addresses will
//! be established to one of the destination addresses in a
//! round-robin manner.
//!
//! For UDP, the packets are sent to the destination ports in a
//! round-robin fashion.
//!

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, str::FromStr};

/// Rule describing where to listen for connections or packets and
/// where to forward the connections or packets.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub protocol: Protocol,
    pub mode: Mode,
    pub source: SocketAddr,
    pub destinations: Vec<SocketAddr>,
}

pub struct Route {
    pub protocol: Protocol,
    pub source: SocketAddr,
    pub destination: SocketAddr,
}

/// Mode for forwarding rule
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Mode {
    RoundRobin,
    Broadcast,
}

/// Protocol
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Protocol {
    Udp,
    Tcp,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseError,
}

/// Storage for state information.
pub struct Database {
    next_id: u32,
    pub rules: HashMap<u32, Rule>,
    pub routes: HashMap<u32, Vec<Route>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            next_id: 0,
            rules: HashMap::new(),
            routes: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.rules.insert(id, rule);
        id
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for Protocol {
    type Err = Error;
    fn from_str(protocol: &str) -> Result<Protocol, <Self as FromStr>::Err> {
        match protocol.to_uppercase().as_str() {
            "UDP" => Ok(Protocol::Udp),
            "TCP" => Ok(Protocol::Tcp),
            _ => Err(Error::ParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol() {
        assert_eq!("udp".parse(), Ok(Protocol::Udp));
        assert_eq!("Udp".parse(), Ok(Protocol::Udp));
        assert_eq!("UDP".parse(), Ok(Protocol::Udp));
        assert_eq!("tcp".parse(), Ok(Protocol::Tcp));
        assert_eq!("Tcp".parse(), Ok(Protocol::Tcp));
        assert_eq!("TCP".parse(), Ok(Protocol::Tcp));
    }
}
