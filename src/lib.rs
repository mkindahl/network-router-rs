// Copyright 2019 Mats Kindahl
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

#[macro_use]
extern crate log;

#[macro_export]
macro_rules! assert_matches {
    ($string:expr, $pattern:expr) => {{
        let re = Regex::new($pattern).unwrap();
        if !re.is_match(&$string.to_string()) {
            panic!(
                "assertion failed: pattern '{}' do not match string '{}'",
                $pattern, $string
            )
        }
    }};
}

pub mod config;
pub mod protocol;
pub mod storage;
pub mod strategy;
