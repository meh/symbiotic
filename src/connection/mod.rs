// Copyleft (ɔ) meh. - http://meh.schizofreni.co
//
// This file is part of symbiotic - https://github.com/meh/symbiotic
//
// symbiotic is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// symbiotic is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with symbiotic. If not, see <http://www.gnu.org/licenses/>.

extern crate regex;

use std::sync::mpsc::{Sender, SendError};

use self::regex::Regex;

use clipboard;

mod incoming;
mod outgoing;

pub struct Broadcast(Vec<Sender<clipboard::Change>>);

impl Broadcast {
	pub fn send(&self, change: clipboard::Change) -> Result<(), SendError<clipboard::Change>> {
		println!("{:?}", change);

		for chan in self.0.iter() {
			if let error@Err(..) = chan.send(change.clone()) {
				return error;
			}
		}

		Ok(())
	}
}

pub fn start(main: Sender<clipboard::Message>, bind: String, port: u16, peers: Vec<String>) -> Broadcast {
	let mut broadcast = vec!();
	let mut ips       = vec!();

	for peer in peers.iter() {
		broadcast.push(outgoing::start(peer.clone()));

		ips.push(Regex::new(r"^(.*?)(:\d+)?$").unwrap().captures(peer.as_slice()).unwrap().at(1).unwrap().to_string());
	}

	incoming::start(main, bind, port, ips);

	Broadcast(broadcast)
}
