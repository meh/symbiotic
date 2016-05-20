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

use std::thread;
use std::sync::mpsc::{Sender, channel};
use std::net::TcpStream;

use openssl::ssl::SslMethod::Sslv23;
use openssl::ssl::{SslContext, SslStream, SSL_VERIFY_PEER};

use protobuf::Message;

use protocol;
use clipboard;
use util;
use connection::Peer;

fn identity() -> protocol::handshake::Identity {
	let mut msg = protocol::handshake::Identity::new();

	msg.set_name("clipboard".to_string());

	msg.mut_version().set_major(env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap());
	msg.mut_version().set_minor(env!("CARGO_PKG_VERSION_MINOR").parse().unwrap());
	msg.mut_version().set_patch(env!("CARGO_PKG_VERSION_PATCH").parse().unwrap());

	msg
}

pub fn start(peer: Peer) -> Sender<clipboard::Change> {
	let (sender, receiver) = channel::<clipboard::Change>();

	thread::spawn(move || {
		loop {
			let conn = wait!(TcpStream::connect((&peer.ip[..], peer.port)));

			debug!("client: connected");

			let mut ctx = SslContext::new(Sslv23).unwrap();
			ctx.set_cipher_list("DEFAULT").unwrap();

			if let Some(ref cert) = peer.cert {
				ctx.set_verify(SSL_VERIFY_PEER, None);
				ctx.set_CA_file(cert).unwrap();
			}

			let mut conn = SslStream::connect(&ctx, conn).unwrap();

			debug!("client: sending handshake");

			if identity().write_length_delimited_to_writer(&mut conn).is_err() {
				continue;
			}

			debug!("client: sent handshake: {:?}", identity());

			util::flush(&receiver);

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
