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

use std::sync::Arc;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};

use self::openssl::ssl::SslMethod::Sslv23;
use self::openssl::ssl::{SslContext, SslStream};
use self::openssl::ssl::SslVerifyMode::SslVerifyPeer;

use clipboard;
use clipboard::Direction::Incoming;

use protocol;

use self::protobuf::Message;
use self::protobuf::core::parse_length_delimited_from;
use self::protobuf::stream::{CodedInputStream, CodedOutputStream};

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

	fn verify(msg: &protocol::handshake::Identity) -> bool {
		if msg.get_name().as_slice() != "clipboard" {
			return false;
		}

		if msg.get_version().get_major() != from_str(env!("CARGO_PKG_VERSION_MAJOR")).unwrap() {
			return false;
		}

		if msg.get_version().get_minor() != from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap() {
			return false;
		}

		if msg.get_version().get_patch() > from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap() {
			return false;
		}

		true
	}

	fn identity() -> protocol::handshake::Identity {
		let mut msg = protocol::handshake::Identity::new();

		msg.set_name("clipboard".to_string());

		msg.mut_version().set_major(from_str(env!("CARGO_PKG_VERSION_MAJOR")).unwrap());
		msg.mut_version().set_minor(from_str(env!("CARGO_PKG_VERSION_MINOR")).unwrap());
		msg.mut_version().set_patch(from_str(env!("CARGO_PKG_VERSION_PATCH")).unwrap());

		return msg;
	}

	pub fn start(&mut self, ipc: Sender<clipboard::Message>) {
		self.broadcast = Some(self.peers.iter().map(|h| {
			let (sender, receiver) = channel::<clipboard::Change>();
			let host               = h.clone();

			spawn(move || {
				loop {
					let mut conn = match TcpStream::connect(host.as_slice()) {
						Ok(conn) =>
							conn,

						Err(_) =>
							continue
					};

					println!("client: connected");

					conn.set_nodelay(true).unwrap();
					conn.close_read().unwrap();

					let mut output = CodedOutputStream::new(&mut conn);

					println!("client: sending handshake");

					if Manager::identity().write_length_delimited_to(&mut output).is_err() {
						continue;
					}

					println!("client: sent handshake: {}", Manager::identity());

					loop {
						println!("client: waiting for message");

						let (ref format, ref data) = *receiver.recv();

						println!("client: message received: {}: {}", format, data);

						{
							let mut msg = protocol::clipboard::Change::new();

							msg.set_format(format.clone());
							msg.set_data(data.clone());

							if msg.write_length_delimited_to(&mut output).is_err() {
								break;
							}
						}

						println!("client: message sent");
					}
				}
			});

			sender
		}).collect());

		let bind = self.bind.clone();
		let port = self.port.clone();

		spawn(move || {
			let     listener = TcpListener::bind((bind.as_slice(), port));
			let mut acceptor = listener.listen();

			for conn in acceptor.incoming() {
				let ipc = ipc.clone();

				match conn {
					Ok(conn) => spawn(move || {
						let mut conn = conn;

						println!("server: connected");

						conn.set_nodelay(true).unwrap();
						conn.close_write().unwrap();

						let mut input = CodedInputStream::new(&mut conn);

						println!("server: fo shizzle");

						match parse_length_delimited_from::<protocol::handshake::Identity>(&mut input) {
							Ok(msg) => {
								if !Manager::verify(&msg) {
									println!("server: handshake invalid");

									return;
								}
							},

							Err(error) => {
								println!("server: handshake error: {}", error);
								return;
							}
						}

						println!("server: handshake verified");

						loop {
							match parse_length_delimited_from::<protocol::clipboard::Change>(&mut input) {
								Ok(mut msg) =>
									ipc.send(Incoming(Arc::new((msg.take_format(), msg.take_data())))),

								Err(_) =>
									break
							}
						}
					}),

					Err(_) => continue
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
