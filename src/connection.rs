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

extern crate openssl;
/*extern crate protobuf;*/

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};

/*use protobuf::stream::{CodedInputStream, CodedOutputStream};*/

use self::openssl::ssl::SslMethod::Sslv23;
use self::openssl::ssl::{SslContext, SslStream};
use self::openssl::ssl::SslVerifyMode::SslVerifyPeer;

use clipboard::Change;

pub struct Manager {
	bind:  String,
	port:  u16,
	hosts: Vec<String>,
}

impl Manager {
	pub fn new(bind: String, port: u16, hosts: Vec<String>) -> Manager {
		Manager { bind: bind, port: port, hosts: hosts }
	}

	pub fn start<F>(&mut self, function: F) where F: Fn(Change) + Send {
		let mut listener = TcpListener::bind((self.bind.as_slice(), self.port));
		let mut acceptor = listener.listen();

		for host in self.hosts.iter().map(|h| h.clone()) {
			spawn(proc() {
				loop {
					let mut conn   = TcpStream::connect(host.as_slice());
					/*let mut stream = CodedInputStream::new(conn);*/
				}
			});
		}

		for stream in acceptor.incoming() {
			match stream {
				Err(e) => {},

				Ok(stream) => spawn(proc() {

				})
			}
		}
	}

	pub fn change(&self, change: Change) {

	}
}
