mod common;

use crate::common::Harness;
use bytes::Buf;
use hyper::{Body, Method, StatusCode};
use router::session::Rule;

const CONFIG: &str = r#"{
  "protocol": "udp",
  "mode": "broadcast",
  "source": "127.0.0.1:8080",
  "destinations": ["127.0.0.1:8081", "127.0.0.1:8082"]
}"#;

const ADD_RULE: &str = r#"{
  "protocol": "udp",
  "mode": "round-robin",
  "source": "127.0.0.1:2345",
   "destinations": ["192.168.1.1:2345", "192.168.1.136:2345"]
}"#;

/// Basic test of JSON get information.
#[test]
fn test_json() {
    let mut harness = Harness::new(Rule::from_json(CONFIG).unwrap(), 2357);

    harness.start().expect("started");

    // Ask for the rules that the harness was configured with. We
    // should find them all.
    {
        let (body, status) = harness
            .send_request(Method::GET, "/rules", Body::default())
            .unwrap();
        let actual_rules: Vec<Rule> = serde_json::from_reader(body.reader()).unwrap();
        let expected_rules = vec![Rule::from_json(CONFIG).unwrap()];
        assert_eq!(status, StatusCode::OK);
        assert_eq!(actual_rules, expected_rules);
    }

    // Try to add one rule and check that it is there when we ask for
    // rules.
    {
        let req_body = Body::from(ADD_RULE);
        let (body, status) = harness
            .send_request(Method::POST, "/rules", req_body)
            .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(br#"{"rule_id":1}"#, body.chunk());
        let (body, status) = harness
            .send_request(Method::GET, "/rules", Body::default())
            .unwrap();
        let expected_rules = vec![
            Rule::from_json(CONFIG).unwrap(),
            Rule::from_json(ADD_RULE).unwrap(),
        ];
        let actual_rules: Vec<Rule> = serde_json::from_reader(body.reader()).unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(actual_rules, expected_rules);
    }
}
