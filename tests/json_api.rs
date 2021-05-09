// Copyright 2019-21 Mats Kindahl
//
// Licensed under the Apache License, Version 2.0 (the "License"); you
// may not use this file except in compliance with the License.  You
// may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.  See the License for the specific language governing
// permissions and limitations under the License.

mod common;

use crate::common::Harness;
use bytes::Buf;
use hyper::{Body, Method, StatusCode};
use router::session::Rule;
use serde::Deserialize;

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

const UPDATE_RULE: &str = r#"{
  "protocol": "udp",
  "mode": "round-robin",
  "source": "127.0.0.1:2345",
  "destinations": ["192.168.1.136:2345"]
}"#;

/// Basic test of JSON get information.
#[test]
fn test_json() {
    let mut harness = Harness::new(Rule::from_json(CONFIG).unwrap(), 2357);
    harness.start().expect("unable to start harness");

    // Ask for the rules that the harness was configured with. We
    // should find them all.
    expect_rules(&mut harness, vec![Rule::from_json(CONFIG).unwrap()]);

    // Try to add one rule and check that it is there when we ask for
    // rules.
    let rule_no = test_add_rule(&mut harness, ADD_RULE);

    // Test updating the added rule
    test_update_rule(&mut harness, rule_no, UPDATE_RULE);

    // Check that deleting the rule actually deletes it.
    test_delete_rule(&mut harness, rule_no);
}

fn test_add_rule(harness: &mut Harness, json: &'static str) -> usize {
    let req_body = Body::from(json);
    let (body, status) = harness
        .send_request(Method::POST, "/rules", req_body)
        .unwrap();
    assert_eq!(status, StatusCode::CREATED);
    let resp: CreateReply = serde_json::from_slice(body.chunk()).unwrap();
    assert_eq!(CreateReply { rule_id: 1 }, resp);
    expect_rules(
        harness,
        vec![
            Rule::from_json(CONFIG).unwrap(),
            Rule::from_json(ADD_RULE).unwrap(),
        ],
    );
    resp.rule_id
}

fn test_delete_rule(harness: &mut Harness, rule_no: usize) {
    let path = format!("/rules/{}", rule_no);
    let (_, status) = harness
        .send_request(Method::DELETE, &path, Body::default())
        .unwrap();
    assert_eq!(status, StatusCode::NO_CONTENT);
    expect_rules(harness, vec![Rule::from_json(CONFIG).unwrap()]);
    let (_, status) = harness
        .send_request(Method::DELETE, &path, Body::default())
        .unwrap();
    assert_eq!(status, StatusCode::NOT_FOUND);
}

fn test_update_rule(harness: &mut Harness, rule_no: usize, json: &'static str) {
    let path = format!("/rules/{}", rule_no);
    let (_, status) = harness
        .send_request(Method::PUT, &path, Body::from(json))
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    expect_rules(
        harness,
        vec![
            Rule::from_json(CONFIG).unwrap(),
            Rule::from_json(UPDATE_RULE).unwrap(),
        ],
    );
}

fn expect_rules(harness: &mut Harness, expected_rules: Vec<Rule>) {
    let (body, status) = harness
        .send_request(Method::GET, "/rules", Body::default())
        .unwrap();
    let actual_rules: Vec<Rule> = serde_json::from_reader(body.reader()).unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(actual_rules, expected_rules);
}

#[derive(Deserialize, PartialEq, Debug)]
struct CreateReply {
    rule_id: usize,
}
