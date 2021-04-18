use crate::session::{Mode, Rule};
use std::{net::SocketAddr, str::FromStr};

pub trait Strategy {
    fn destinations(&mut self) -> Vec<SocketAddr>;
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseModeError(String),
}

pub struct StrategyFactory;

impl StrategyFactory {
    /// Create a boxed strategy based on a mode and a vector of
    /// destinations.
    pub fn make(rule: &Rule) -> Box<dyn Strategy + Send> {
        match rule.mode {
            Mode::Broadcast => Box::new(BroadcastStrategy::new(&rule.destinations)),
            Mode::RoundRobin => Box::new(RoundRobinStrategy::new(&rule.destinations)),
        }
    }
}

/// Strategy for broadcasting packets to all destinations. Only makes
/// sense for UDP.
#[derive(Clone)]
pub struct BroadcastStrategy {
    peers: Vec<SocketAddr>,
}

/// Strategy for sending packets or connections to destinations
/// one-by-one.
#[derive(Clone)]
pub struct RoundRobinStrategy {
    next: usize,
    peers: Vec<SocketAddr>,
}

impl BroadcastStrategy {
    pub fn new(peers: &[SocketAddr]) -> BroadcastStrategy {
        debug!("Broadcast strategy with peers {:?}", peers);
        BroadcastStrategy {
            peers: peers.to_owned(),
        }
    }
}

impl RoundRobinStrategy {
    pub fn new(peers: &[SocketAddr]) -> RoundRobinStrategy {
        debug!("RoundRobin strategy with peers {:?}", peers);
        RoundRobinStrategy {
            next: 0,
            peers: peers.to_owned(),
        }
    }
}

impl Strategy for BroadcastStrategy {
    fn destinations(&mut self) -> Vec<SocketAddr> {
        self.peers.clone()
    }
}

impl Strategy for RoundRobinStrategy {
    fn destinations(&mut self) -> Vec<SocketAddr> {
        let result = vec![self.peers[self.next]];
        self.next += 1;
        if self.next >= self.peers.len() {
            self.next = 0;
        }
        result
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseModeError(ref txt) => write!(f, "{} is not a mode", txt),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::ParseModeError(_) => "mode error",
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::RoundRobin => write!(f, "RoundRobin"),
            Mode::Broadcast => write!(f, "Broadcast"),
        }
    }
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("roundrobin") {
            Ok(Mode::RoundRobin)
        } else if s.eq_ignore_ascii_case("broadcast") {
            Ok(Mode::Broadcast)
        } else {
            Err(Error::ParseModeError(s.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode() {
        assert_eq!("roundrobin".parse(), Ok(Mode::RoundRobin));
        assert_eq!("broadcast".parse(), Ok(Mode::Broadcast));
    }
}
