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
use std::thread::Thread;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};

use std::io::timer::sleep;
use std::time::duration::Duration;

use self::openssl::ssl::SslMethod::Sslv23;
use self::openssl::ssl::{SslContext, SslStream};
use self::openssl::ssl::SslVerifyMode::SslVerifyPeer;

use protocol;

use protobuf::{Message, RepeatedField};
use protobuf::core::parse_length_delimited_from;
use protobuf::stream::{CodedInputStream};

use clipboard;
use clipboard::Direction::Incoming;

fn verify(msg: &protocol::handshake::Identity) -> bool {
	if msg.get_name().as_slice() != "clipboard" {
		return false;
	}

	if msg.get_version().get_major() != env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap() {
		return false;
	}

	if msg.get_version().get_minor() != env!("CARGO_PKG_VERSION_MINOR").parse().unwrap() {
		return false;
	}

	if msg.get_version().get_patch() > env!("CARGO_PKG_VERSION_MINOR").parse().unwrap() {
		return false;
	}

	true
}

pub fn start(main: Sender<clipboard::Message>, bind: String, port: u16, peers: Vec<String>) {
	Thread::spawn(move || -> () {
		let     listener = TcpListener::bind((bind.as_slice(), port));
		let mut acceptor = listener.listen();

		for conn in acceptor.incoming() {
			let main = main.clone();

			if let Ok(conn) = conn {
				Thread::spawn(move || -> () {
					let mut conn = conn;

					debug!("server: connected");

					conn.set_nodelay(true).unwrap();
					conn.close_write().unwrap();

					let mut input = CodedInputStream::new(&mut conn);

					debug!("server: fo shizzle");

					match parse_length_delimited_from::<protocol::handshake::Identity>(&mut input) {
						Ok(msg) => {
							if !verify(&msg) {
								debug!("server: handshake invalid");

								return;
							}
						},

						Err(error) => {
							debug!("server: handshake error: {}", error);
							return;
						}
					}

					debug!("server: handshake verified");

					loop {
						match parse_length_delimited_from::<protocol::clipboard::Change>(&mut input) {
							Ok(mut msg) => {
								debug!("server: received {}", msg);
								//ipc.send(Incoming(Arc::new((msg.get_id(), msg.take_format(), msg.take_data()))));
							},

							Err(_) => {
								break;
							}
						}
					}
				}).detach()
			}
			else {
				continue
			}
		}
	}).detach();
}
