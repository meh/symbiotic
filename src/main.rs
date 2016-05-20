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

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
use clap::{Arg, App};

extern crate toml;
extern crate tempdir;

extern crate regex;
use regex::Regex;

extern crate protobuf;
extern crate openssl;

extern crate libc;

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;

#[macro_use]
pub mod util;

pub mod protocol;
pub mod connection;
use connection::Peer;

pub mod clipboard;
use clipboard::Message;
use clipboard::Direction::{Incoming, Outgoing};

pub mod platform;

#[derive(PartialEq, Eq)]
enum Mode {
	Both,
	Incoming,
	Outgoing,
}

fn main() {
	env_logger::init().unwrap();

	let matches = App::new("symbiotic-clipboard")
		.version("0.1")
		.author("meh <meh@schizofreni.co>")
		.about("Clipboard sharing.")
			.arg(Arg::with_name("peers")
				.help("List of peers.")
				.index(1)
				.multiple(true))
			.arg(Arg::with_name("bind")
				.help("IP to bind on (default 0.0.0.0).")
				.short("b")
				.long("bind")
				.takes_value(true))
			.arg(Arg::with_name("port")
				.help("Port to listen on (default 23421)")
				.short("p")
				.long("port")
				.takes_value(true))
			.arg(Arg::with_name("config")
				.help("Path to the config file.")
				.short("c")
				.long("config")
				.takes_value(true))
			.arg(Arg::with_name("limit")
				.help("Maximum size of the clipboard data.")
				.short("l")
				.long("limit")
				.takes_value(true))
			.arg(Arg::with_name("filter")
				.help("List of ':' separated mime types to ignore.")
				.short("f")
				.long("filter")
				.takes_value(true))
			.arg(Arg::with_name("incoming")
				.help("Only receive clipboard changes.")
				.short("i")
				.long("incoming"))
			.arg(Arg::with_name("outgoing")
				.help("Only send clipboard changes.")
				.short("o")
				.long("outgoing"))
			.get_matches();

	let mut peers:    Vec<Peer>           = vec!();
	let mut host:     Peer                = Default::default();
	let mut platform: Option<toml::Value> = None;
	let mut mode:     Mode                = Mode::Both;
	let mut limit:    usize               = 0;
	let mut filter:   Vec<String>         = vec!();

	if let Some(path) = matches.value_of("config") {
		let config = util::file_contents(path).expect("config: file not found");
		let config = toml::Parser::new(&config).parse().expect("config: parse failed");

		if let Some(value) = config.get("mode") {
			mode = match value.as_str().unwrap() {
				"both"     => Mode::Both,
				"incoming" => Mode::Incoming,
				"outgoing" => Mode::Outgoing,

				n => panic!("config: unknown mode: {}", n)
			};
		}

		if let Some(value) = config.get("ip") {
			host.ip = value.as_str().unwrap().to_string();
		}

		if let Some(value) = config.get("port") {
			host.port = value.as_integer().unwrap() as u16;
		}

		if let Some(value) = config.get("cert") {
			host.cert = Some(PathBuf::from(value.as_str().unwrap()));
		}

		if let Some(value) = config.get("key") {
			host.key = Some(PathBuf::from(value.as_str().unwrap()));
		}

		if let Some(value) = config.get("limit") {
			limit = human(value.as_str().unwrap())
		}

		if let Some(value) = config.get("filter") {
			filter = value.as_str().unwrap().split(':').map(|s| s.to_string()).collect();
		}

		if let Some(table) = config.get("connection") {
			for value in table.as_table().unwrap().values() {
				let     table      = value.as_table().unwrap();
				let mut peer: Peer = Default::default();

				if let Some(value) = table.get("ip") {
					peer.ip = value.as_str().unwrap().to_string();
				}

				if let Some(value) = table.get("port") {
					peer.port = value.as_integer().unwrap() as u16;
				}

				if let Some(value) = table.get("cert") {
					peer.cert = Some(PathBuf::from(value.as_str().unwrap()));
				}

				peers.push(peer);
			}
		}

		platform = config.get("platform").map(|p| p.clone());

	}
	else {
		if let Some(value) = matches.value_of("port") {
			host.port = value.parse().unwrap();
		}

		if let Some(value) = matches.value_of("bind") {
			host.ip = value.to_string();
		}

		if let Some(value) = matches.value_of("limit") {
			limit = human(value);
		}

		if let Some(value) = matches.value_of("filter") {
			filter = value.split(':').map(|s| s.to_string()).collect();
		}

		if matches.is_present("incoming") {
			mode = Mode::Incoming;
		}

		if matches.is_present("outgoing") {
			mode = Mode::Outgoing;
		}

		for string in matches.values_of("peers").map(|v| v.collect()).unwrap_or(Vec::new()) {
			let mut peer: Peer = Default::default();
			let mut parts      = string.split(':');

			if let Some(ip) = parts.next() {
				peer.ip = ip.to_string();
			}

			if let Some(port) = parts.next() {
				peer.port = port.parse().unwrap();
			}

			peers.push(peer);
		}
	}

	let (sender, receiver) = channel::<Message>();
	let connection         = connection::start(sender.clone(), host, peers);
	let clipboard          = platform::start(sender.clone(), platform);
	let filter             = filter.into_iter().map(|f| wildcard(&f)).collect::<Vec<Regex>>();

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
						let &mut (_, ref mut content) = Arc::get_mut(&mut change).unwrap();

						if !filter.is_empty() {
							for regex in &filter {
								content.retain(|&(ref mime, _)| !regex.is_match(&mime));
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

fn human(string: &str) -> usize {
	let c = Regex::new(r"^(\d+)(KMGT)?$").unwrap().captures(string).unwrap();
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
	let string = Regex::new(r"([\\+*?[^\]$(){}=!<>|:-])").unwrap().replace(string, "\\$1");
	let string = Regex::new(r"\\\*").unwrap().replace(&string, ".*?");
	let string = Regex::new(r"\\\?").unwrap().replace(&string, ".");
	let string = format!("^{}$", string);

	Regex::new(&string).unwrap()
}
