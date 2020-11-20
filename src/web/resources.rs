//! Parse and handle resources in the web server.

use super::error::{Error, Result};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub(crate) enum Resource {
    Rule(Option<u32>),
    Route(u32, Option<u32>),
}

impl Resource {
    pub(crate) fn from_path(path: &[&str]) -> Result<Resource> {
        if path.len() > 0 {
            match path[0] {
                "rules" => parse_rules(&path[1..]),
                _ => Err(Error::ResourceNotFound),
            }
        } else {
            Err(Error::ResourceNotFound)
        }
    }
}

impl FromStr for Resource {
    type Err = Error;

    fn from_str(path: &str) -> Result<Resource> {
        let path: Vec<&str> = path.split("/").collect();
        Resource::from_path(&path)
    }
}

fn parse_rules(path: &[&str]) -> Result<Resource> {
    if path.len() > 0 {
        let rule_no: u32 = path[0].parse().map_err(|_| Error::ResourceNotFound)?;
        parse_rule(rule_no, &path[1..])
    } else {
        Ok(Resource::Rule(None))
    }
}

fn parse_rule(rule_no: u32, path: &[&str]) -> Result<Resource> {
    if path.len() > 0 {
        match path[0] {
            "routes" => parse_routes(rule_no, &path[1..]),
            _ => Err(Error::ResourceNotFound),
        }
    } else {
        Ok(Resource::Rule(Some(rule_no)))
    }
}

fn parse_routes(rule_no: u32, path: &[&str]) -> Result<Resource> {
    if path.len() > 0 {
        let route_no: u32 = path[0].parse().map_err(|_| Error::ResourceNotFound)?;
        Ok(Resource::Route(rule_no, Some(route_no)))
    } else {
        Ok(Resource::Route(rule_no, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules() {
        assert_eq!(Resource::from_path(&[]), Err(Error::ResourceNotFound));
        assert_eq!(Resource::from_path(&["rules"]), Ok(Resource::Rule(None)));
        assert_eq!(
            Resource::from_path(&["rules", "123"]),
            Ok(Resource::Rule(Some(123u32)))
        );
        assert_eq!(
            Resource::from_path(&["rules", "foo"]),
            Err(Error::ResourceNotFound)
        );
    }

    #[test]
    fn test_routes() {
        assert_eq!(
            Resource::from_path(&["rules", "123", "routes", "32"]),
            Ok(Resource::Route(123, Some(32)))
        );
        assert_eq!(
            Resource::from_path(&["rules", "123", "routes"]),
            Ok(Resource::Route(123, None))
        );
    }
}
