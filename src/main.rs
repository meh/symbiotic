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

#![feature(plugin)]
#![allow(unstable)]

extern crate protobuf;
extern crate "rustc-serialize" as rustc_serialize;
extern crate toml;
extern crate docopt;
#[plugin] extern crate docopt_macros;
#[macro_use] extern crate log;

use std::io::File;
use std::num::ToPrimitive;

use std::sync::mpsc::channel;

use clipboard::Message;
use clipboard::Direction::{Incoming, Outgoing};

mod connection;
mod platform;
mod clipboard;
mod protocol;

docopt!(Args derive Show, "
Usage: symbiotic-clipboard (-c PATH | --config PATH)
       symbiotic-clipboard [options] <peers>...
       symbiotic-clipboard --help

Options:
  -h, --help         Show this message.
  -b, --bind IP      IP to bind on (default 0.0.0.0).
  -p, --port PORT    Port to listen on (default 23421).
  -c, --config PATH  Path to the config file.

  -i, --incoming     Only receive clipboard changes.
  -o, --outgoing     Only send clipboard changes.
", flag_bind: Option<String>, flag_port: Option<u16>, flag_config: Option<String>,
   arg_peers: Option<Vec<String>>);

fn main() {
	#[derive(PartialEq, Eq)]
	enum Mode {
		Both,
		Incoming,
		Outgoing,
	}

	let mut peers:    Vec<String>         = vec!();
	let mut bind:     String              = "0.0.0.0".to_string();
	let mut port:     u16                 = 23421;
	let mut platform: Option<toml::Value> = None;
	let mut mode:     Mode                = Mode::Both;

	let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

	if let Some(path) = args.flag_config {
		let config = match File::open(&Path::new(path.as_slice())).read_to_string().ok() {
			Some(content) =>
				toml::Parser::new(content.as_slice()).parse().unwrap(),

			None =>
				panic!("{}: file not found", path)
		};

		if let Some(h) = config.get("peers") {
			peers = h.as_slice().unwrap().iter()
			         .map(|x| x.as_str().unwrap().to_string())
			         .collect::<Vec<String>>();
		}

		if let Some(p) = config.get("port") {
			port = p.as_integer().unwrap().to_u16().unwrap();
		}

		if let Some(b) = config.get("bind") {
			bind = b.as_str().unwrap().to_string();
		}

		if let Some(m) = config.get("mode") {
			mode = match m.as_str().unwrap() {
				"both"     => Mode::Both,
				"incoming" => Mode::Incoming,
				"outgoing" => Mode::Outgoing,

				n => panic!("{}: unknown mode", n)
			};
		}

		platform = config.get("platform").map(|p| p.clone());
	}
	else {
		if let Some(p) = args.arg_peers {
			peers = p;
		}

		if let Some(p) = args.flag_port {
			port = p;
		}

		if let Some(b) = args.flag_bind {
			bind = b;
		}

		if args.flag_incoming {
			mode = Mode::Incoming;
		}

		if args.flag_outgoing {
			mode = Mode::Outgoing;
		}
	}

	let (sender, receiver) = channel::<Message>();
	let connection         = connection::start(sender.clone(), bind, port, peers);
	let clipboard          = platform::start(sender.clone(), platform);

	loop {
		match receiver.recv().unwrap() {
			Incoming(change) => {
				if mode != Mode::Outgoing {
					clipboard.send(change).unwrap();
				}
			},

			Outgoing(change) => {
				if mode != Mode::Incoming {
					connection.send(change).unwrap();
				}
			}
		}
	}
}
