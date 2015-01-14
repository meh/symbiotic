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
use std::default::Default;

use clipboard;

mod incoming;
mod outgoing;

#[derive(Clone)]
pub struct Peer {
	pub ip:   String,
	pub port: u16,

	pub cert: Option<Path>,
	pub key:  Option<Path>,
}

impl Default for Peer {
	fn default() -> Self {
		Peer {
			ip:   "0.0.0.0".to_string(),
			port: 23421,

			cert: None,
			key:  None,
		}
	}
}

pub struct Broadcast(Vec<Sender<clipboard::Change>>);

impl Broadcast {
	pub fn send(&self, change: clipboard::Change) -> Result<(), SendError<clipboard::Change>> {
		debug!("broadcast: {:?}", change);

		for chan in self.0.iter() {
			try!(chan.send(change.clone()));
		}

		Ok(())
	}
}

pub fn start(main: Sender<clipboard::Message>, host: Peer, peers: Vec<Peer>) -> Broadcast {
	let mut broadcast = vec!();

	for peer in peers.iter() {
		broadcast.push(outgoing::start(peer.clone()));
	}

	incoming::start(main, host, peers);

	Broadcast(broadcast)
}
