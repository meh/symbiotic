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
use std::sync::mpsc::{Sender, channel};

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};

use std::io::timer::sleep;
use std::time::duration::Duration;

use self::openssl::ssl::SslMethod::Sslv23;
use self::openssl::ssl::{SslContext, SslStream};
use self::openssl::ssl::SslVerifyMode::SslVerifyPeer;

use protocol;

use self::protobuf::{Message, RepeatedField};
use self::protobuf::core::parse_length_delimited_from;
use self::protobuf::stream::{CodedInputStream};

use clipboard;

fn identity() -> protocol::handshake::Identity {
	let mut msg = protocol::handshake::Identity::new();

	msg.set_name("clipboard".to_string());

	msg.mut_version().set_major(env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap());
	msg.mut_version().set_minor(env!("CARGO_PKG_VERSION_MINOR").parse().unwrap());
	msg.mut_version().set_patch(env!("CARGO_PKG_VERSION_PATCH").parse().unwrap());

	msg
}

pub fn start(host: String) -> Sender<clipboard::Change> {
	let (sender, receiver) = channel::<clipboard::Change>();

	Thread::spawn(move || -> () {
		loop {
			let mut conn = match TcpStream::connect(host.as_slice()) {
				Ok(conn) => {
					conn
				},

				Err(_) => {
					sleep(Duration::seconds(1));
					continue
				},
			};

			debug!("client: connected");

			conn.set_nodelay(true).unwrap();
			conn.close_read().unwrap();

			debug!("client: sending handshake");

			if identity().write_length_delimited_to_writer(&mut conn).is_err() {
				continue;
			}

			debug!("client: sent handshake: {:?}", identity());

			loop {
				debug!("client: waiting for message");

				let (ref at, ref content) = *receiver.recv().unwrap();

				debug!("client: message received: @{:?} {:?}", at, content);

				{
					let mut msg = protocol::clipboard::Change::new();

					msg.set_at(*at);
					msg.set_content(content.iter().map(|&(ref format, ref data)| {
						let mut msg = protocol::clipboard::Content::new();

						msg.set_format(format.clone());
						msg.set_data(data.clone());

						msg
					}).collect());

					if msg.write_length_delimited_to_writer(&mut conn).is_err() {
						break;
					}
				}

				debug!("client: message sent");
			}
		}
	});

	sender
}
