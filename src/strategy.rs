use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    RoundRobin,
    Broadcast,
}

#[derive(Clone)]
pub struct Strategy {
    mode: Mode,
    next: usize,
    peers: Vec<SocketAddr>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseModeError(String),
}

impl Strategy {
    pub fn new(mode: Mode, peers: &[SocketAddr]) -> Strategy {
        debug!("strategy {} with peers {:?}", mode, peers);
        Strategy {
            mode,
            next: 0,
            peers: peers.to_owned(),
        }
    }

    pub fn destinations(&mut self) -> Vec<SocketAddr> {
        match self.mode {
            Mode::Broadcast => self.peers.clone(),
            Mode::RoundRobin => {
                let result = vec![self.peers[self.next]];
                self.next += 1;
                if self.next >= self.peers.len() {
                    self.next = 0;
                }
                result
            }
        }
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
