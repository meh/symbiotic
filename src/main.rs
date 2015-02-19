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

#![feature(plugin, core, collections, alloc, std_misc, libc, hash, unicode, link_args, old_io, old_path)]
#![plugin(docopt_macros, regex_macros)]
#![allow(unused_features)]

extern crate protobuf;

extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;

extern crate toml;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate regex;

use std::old_io::File;
use std::num::ToPrimitive;

use std::sync::mpsc::channel;

use regex::Regex;

use clipboard::Message;
use clipboard::Direction::{Incoming, Outgoing};

use connection::Peer;
use std::default::Default;

mod utils;
mod connection;
mod platform;
mod clipboard;
mod protocol;

docopt!(Args derive Debug, "
Usage: symbiotic-clipboard (-c PATH | --config PATH)
       symbiotic-clipboard [options] <peers>...
       symbiotic-clipboard --help

Options:
  -h, --help         Show this message.
  -b, --bind IP      IP to bind on (default 0.0.0.0).
  -p, --port PORT    Port to listen on (default 23421).
  -c, --config PATH  Path to the config file.

  -l, --limit SIZE   Maximum size of the clipboard data to send.
  -f, --filter MIME  List of ':' separated mime types to ignore.

  -i, --incoming     Only receive clipboard changes.
  -o, --outgoing     Only send clipboard changes.
", flag_bind: Option<String>, flag_port: Option<u16>, flag_config: Option<String>,
   flag_limit: Option<String>, flag_filter: Option<String>,
   arg_peers: Option<Vec<String>>);

fn human(string: &str) -> usize {
	let c = regex!(r"^(\d+)(KMGT)?$").captures(string).unwrap();
	let n = c.at(0).unwrap().parse::<usize>().unwrap();

	match c.at(1) {
		None      => n,
		Some("K") => n * 1024,
		Some("M") => n * 1024 * 1024,
		Some("G") => n * 1024 * 1024 * 1024,
		Some("T") => n * 1024 * 1024 * 1024 * 1024,

		_ => panic!("unknown size")
	}
}

fn wildcard(string: &str) -> Regex {
	let string = regex!(r"([\\+*?[^\]$(){}=!<>|:-])").replace(string, "\\$1");
	let string = regex!(r"\\\*").replace(&string[], ".*?");
	let string = regex!(r"\\\?").replace(&string[], ".");
	let string = format!("^{}$", string);

	Regex::new(&string[]).unwrap()
}

fn main() {
	#[derive(PartialEq, Eq)]
	enum Mode {
		Both,
		Incoming,
		Outgoing,
	}

	let mut peers:    Vec<Peer>           = vec!();
	let mut host:     Peer                = Default::default();
	let mut platform: Option<toml::Value> = None;
	let mut mode:     Mode                = Mode::Both;
	let mut limit:    usize               = 0;
	let mut filter:   Vec<String>         = vec!();

	let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

	if let Some(path) = args.flag_config {
		let config = match File::open(&Path::new(&path[])).read_to_string() {
			Ok(content) =>
				toml::Parser::new(&content[]).parse().unwrap(),

			Err(..) =>
				panic!("{}: file not found", path)
		};

		if let Some(m) = config.get("mode") {
			mode = match m.as_str().unwrap() {
				"both"     => Mode::Both,
				"incoming" => Mode::Incoming,
				"outgoing" => Mode::Outgoing,

				n => panic!("{}: unknown mode", n)
			};
		}

		if let Some(ip) = config.get("ip") {
			host.ip = ip.as_str().unwrap().to_string();
		}

		if let Some(port) = config.get("port") {
			host.port = port.as_integer().unwrap().to_u16().unwrap();
		}

		if let Some(cert) = config.get("cert") {
			host.cert = Some(Path::new(cert.as_str().unwrap()));
		}

		if let Some(key) = config.get("key") {
			host.key = Some(Path::new(key.as_str().unwrap()));
		}

		if let Some(l) = config.get("limit") {
			limit = human(l.as_str().unwrap())
		}

		if let Some(f) = config.get("filter") {
			filter = f.as_str().unwrap().split(':').map(|s| s.to_string()).collect();
		}

		if let Some(table) = config.get("connection") {
			for value in table.as_table().unwrap().values() {
				let     table      = value.as_table().unwrap();
				let mut peer: Peer = Default::default();

				if let Some(ip) = table.get("ip") {
					peer.ip = ip.as_str().unwrap().to_string();
				}

				if let Some(port) = table.get("port") {
					peer.port = port.as_integer().unwrap().to_u16().unwrap();
				}

				if let Some(cert) = table.get("cert") {
					peer.cert = Some(Path::new(cert.as_str().unwrap()));
				}

				peers.push(peer);
			}
		}

		platform = config.get("platform").map(|p| p.clone());
	}
	else {
		if let Some(p) = args.flag_port {
			host.port = p;
		}

		if let Some(b) = args.flag_bind {
			host.ip = b;
		}

		if let Some(p) = args.arg_peers {
			for string in p.iter() {
				let mut peer: Peer = Default::default();
				let mut parts      = (&string[]).split(':');

				if let Some(ip) = parts.next() {
					peer.ip = ip.to_string();
				}

				if let Some(port) = parts.next() {
					peer.port = port.parse().unwrap();
				}

				peers.push(peer);
			}
		}

		if let Some(l) = args.flag_limit {
			limit = human(&l[])
		}

		if let Some(f) = args.flag_filter {
			filter = f.split(':').map(|s| s.to_string()).collect();
		}

		if args.flag_incoming {
			mode = Mode::Incoming;
		}

		if args.flag_outgoing {
			mode = Mode::Outgoing;
		}
	}

	env_logger::init().unwrap();

	let (sender, receiver) = channel::<Message>();
	let connection         = connection::start(sender.clone(), host, peers);
	let clipboard          = platform::start(sender.clone(), platform);

	loop {
		match receiver.recv().unwrap() {
			Incoming(change) => {
				if mode != Mode::Outgoing {
					clipboard.send(change).unwrap();
				}
			},

			Outgoing(mut change) => {
				if mode != Mode::Incoming {
					{
						let &mut (_, ref mut content) = change.make_unique();

						if !filter.is_empty() {
							for wc in filter.iter() {
								let wc = wildcard(&wc[]);

								content.retain(|&(ref mime, _)| !wc.is_match(&mime[]));
							}
						}

						if limit > 0 {
							content.retain(|&(_, ref content)| content.len() < limit);
						}
					}

					connection.send(change).unwrap();
				}
			}
		}
	}
}
