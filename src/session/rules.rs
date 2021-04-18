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
use std::{mem, net::SocketAddr};

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
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    RoundRobin,
    Broadcast,
}

/// Protocol
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
    pub rules: Vec<Option<Rule>>,
}

impl Database {
    pub fn new() -> Self {
        Database { rules: Vec::new() }
    }

    /// Create a new rule.
    pub fn create_rule(&mut self, rule: Rule) -> usize {
        let id = self.rules.len();
        self.rules.push(Some(rule));
        id
    }

    /// Remove an existing rule, if it exists.
    pub fn drop_rule(&mut self, id: usize) -> Option<Rule> {
        mem::replace(&mut self.rules[id], None)
    }

    /// Update an existing rule, if it exists.
    pub fn update_rule(&mut self, id: usize, rule: Rule) -> Option<Rule> {
        mem::replace(&mut self.rules[id], Some(rule))
    }

    /// Get rule from rule identifier.
    pub fn get_rule(&self, id: usize) -> Option<&Rule> {
        self.rules.get(id).unwrap_or(&None).as_ref()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
