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

extern crate serialize;

extern crate toml;

extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;

use std::io::File;
use clipboard::Clipboard;

mod connection;
mod platform;
mod clipboard;

docopt!(Args deriving Show, "
Usage: symbiotic-clipboard (-c PATH | --config PATH)
       symbiotic-clipboard [options] <hosts>...
       symbiotic-clipboard --help

Options:
  -h, --help         Show this message.
  -p, --port PORT    Port to listen on.
  -c, --config PATH  Path to the config file.
", flag_port: Option<u16>, flag_config: Option<String>, arg_hosts: Option<Vec<String>>)

fn main() {
	let hosts: Vec<String>;
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

			hosts = match config.get("hosts") {
				Some(h) => h.as_slice().unwrap().iter()
				            .map(|x| x.as_str().unwrap().to_string())
				            .collect::<Vec<String>>(),

				None => vec!()
			};

			port = match config.get("port") {
				Some(p) => p.as_integer().unwrap().to_u16().unwrap(),
				None    => 23421
			};

			specs = config.get("platform").map(|p| p.clone());
		}

		None => {
			hosts = args.arg_hosts.unwrap_or(vec!());
			port  = args.flag_port.unwrap_or(23421);
			specs = None;
		}
	}

	let mut manager   = connection::Manager::new(port, hosts);
	let mut clipboard = platform::Clipboard::new(specs);
	
	clipboard.start(|change| {
		manager.change(change);
	});

	manager.start(|change| {
		clipboard.set(change);
	});
}
