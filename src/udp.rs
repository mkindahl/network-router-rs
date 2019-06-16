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

struct UdpSession {
    socket: UdpSocket,
    peers: Vec<SocketAddr>,
}

impl UdpSession {
    fn new(source: &SocketAddr, destinations: &Vec<SocketAddr>) -> std::io::Result<UdpSession> {
        let mut peers: Vec<SocketAddr> = Vec::new();
        for dest in destinations {
            peers.push(*dest);
        }
        let session = UdpSession {
            socket: UdpSocket::bind(&source)?,
            peers: peers,
        };
        Ok(session)
    }
}
