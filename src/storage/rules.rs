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
use std::{collections::HashMap, net::SocketAddr};

/// Rule describing where to listen for connections or packets and
/// where to forward the connections or packets.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub protocol: Protocol,
    pub mode: Mode,
    pub source: SocketAddr,
    pub destinations: Vec<SocketAddr>,
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

/// Storage for state information.
pub struct Database {
    next_id: i32,
    pub rules: HashMap<i32, Rule>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            next_id: 0,
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.insert(self.next_id, rule);
        self.next_id += 1;
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
