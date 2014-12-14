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
extern crate protobuf;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};

use std::io::timer::sleep;
use std::time::duration::Duration;

use self::protobuf::stream::{CodedInputStream, CodedOutputStream};

use self::openssl::ssl::SslMethod::Sslv23;
use self::openssl::ssl::{SslContext, SslStream};
use self::openssl::ssl::SslVerifyMode::SslVerifyPeer;

use clipboard;
use clipboard::Direction::Incoming;

use protocol;

use self::protobuf::Message;
use self::protobuf::stream::{WithCodedInputStream, WithCodedOutputStream};

#[deriving(Clone)]
pub struct Manager {
	bind:  String,
	port:  u16,
	peers: Vec<String>,

	broadcast: Option<Vec<Sender<clipboard::Change>>>,
}

impl Manager {
	pub fn new(bind: String, port: u16, peers: Vec<String>) -> Manager {
		Manager { bind: bind, port: port, peers: peers, broadcast: None }
	}

	pub fn start(&mut self, ipc: Sender<clipboard::Message>) {
		self.broadcast = Some(self.peers.iter().map(|h| {
			let (sender, receiver) = channel();
			let host               = h.clone();

			spawn(proc() {
				loop {
					let mut conn = match TcpStream::connect(host.as_slice()) {
						Ok(conn) => conn,
						Err(_)   => continue
					};

					{
						let mut msg = protocol::handshake::Identity::new();

						msg.set_name("clipboard".to_string());

						msg.mut_version().set_major(from_str(env!("CARGO_PKG_VERSION_MAJOR")).unwrap());
						msg.mut_version().set_minor(from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap());
						msg.mut_version().set_patch(from_str(env!("CARGO_PKG_VERSION_PATCH")).unwrap());

						if msg.write_to_writer(&mut conn).is_err() {
							continue;
						}
					}

					match protobuf::parse_from_reader::<protocol::handshake::Identity>(&mut conn) {
						Ok(msg) => {
							if msg.get_name().as_slice() != "clipboard" {
								continue;
							}

							if msg.get_version().get_major() != from_str(env!("CARGO_PKG_VERSION_MAJOR")).unwrap() {
								continue;
							}

							if msg.get_version().get_minor() != from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap() {
								continue;
							}

							if msg.get_version().get_patch() > from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap() {
								continue;
							}
						},

						Err(_) => continue
					}

					loop {
						let     (format, data) = receiver.recv();
						let mut msg            = protocol::clipboard::Change::new();
					}
				}
			});

			sender
		}).collect());


		let bind = self.bind.clone();
		let port = self.port.clone();

		spawn(proc() {
			let mut listener = TcpListener::bind((bind.as_slice(), port));
			let mut acceptor = listener.listen();

			for stream in acceptor.incoming() {
				match stream {
					Err(e) => {},

					Ok(stream) => spawn(proc() {

					})
				}
			}
		});
	}

	pub fn set(&self, change: clipboard::Change) {
		for sender in self.broadcast.as_ref().unwrap().iter() {
			sender.send(change.clone());
		}
	}
}
