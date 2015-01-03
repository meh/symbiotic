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

#![feature(phase)]

// XXX: remove me when done prototyping
#![allow(dead_code, unused_variables, unused_imports)]

extern crate protobuf;
extern crate "rustc-serialize" as rustc_serialize;
extern crate toml;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin, link)] extern crate log;

use std::io::File;

use clipboard::{Change, Message};
use clipboard::Direction::{Incoming, Outgoing};

mod connection;
mod platform;
mod clipboard;
mod protocol;

docopt!(Args deriving Show, "
Usage: symbiotic-clipboard (-c PATH | --config PATH)
       symbiotic-clipboard [options] <peers>...
       symbiotic-clipboard --help

Options:
  -h, --help         Show this message.
  -b, --bind IP      IP to bind on (default 0.0.0.0).
  -p, --port PORT    Port to listen on (default 23421).
  -c, --config PATH  Path to the config file.
", flag_bind: Option<String>, flag_port: Option<u16>, flag_config: Option<String>,
   arg_peers: Option<Vec<String>>);

fn main() {
	let peers: Vec<String>;
	let bind:  String;
	let port:  u16;
	let specs: Option<toml::Value>;

	let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

	match args.flag_config {
		Some(path) => {
			let config = match File::open(&Path::new(path.as_slice())).read_to_string().ok() {
				Some(content) => 
					toml::Parser::new(content.as_slice()).parse().unwrap(),

				None =>
					panic!("{}: file not found", path)
			};

			peers = match config.get("peers") {
				Some(h) => h.as_slice().unwrap().iter()
				            .map(|x| x.as_str().unwrap().to_string())
				            .collect::<Vec<String>>(),

				None => vec!()
			};

			port = match config.get("port") {
				Some(p) => p.as_integer().unwrap().to_u16().unwrap(),
				None    => 23421
			};

			bind = match config.get("bind") {
				Some(b) => b.as_str().unwrap().to_string(),
				None    => "0.0.0.0".to_string()
			};

			specs = config.get("platform").map(|p| p.clone());
		}

		None => {
			peers = args.arg_peers.unwrap_or(vec!());
			port  = args.flag_port.unwrap_or(23421);
			bind  = args.flag_bind.unwrap_or("0.0.0.0".to_string());
			specs = None;
		}
	}

	let (sender, receiver) = channel::<Message>();
	let connection         = connection::start(sender.clone(), bind, port, peers);
	let clipboard          = platform::start(sender.clone(), specs);
	
	loop {
		match receiver.recv() {
			Incoming(change) => {
				clipboard.send(change);
			},

			Outgoing(change) => {
				connection.send(change);
			}
		}
	}
}
