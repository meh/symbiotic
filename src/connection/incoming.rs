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
use std::sync::mpsc::Sender;

use std::io::TcpListener;
use std::io::{Acceptor, Listener};
use std::io::{File, Open, Write};
use std::io::TempDir;

use self::openssl::ssl::SslMethod::Sslv23;
use self::openssl::ssl::{SslContext, SslStream};
use self::openssl::ssl::SslVerifyMode::{SslVerifyNone, SslVerifyPeer};
use self::openssl::x509::{X509FileType, X509Generator};
use self::openssl::x509::KeyUsage::DigitalSignature;
use self::openssl::crypto::hash::HashType::SHA256;

use protocol;

use protobuf::Message;
use protobuf::core::parse_length_delimited_from;
use protobuf::stream::{CodedInputStream};

use clipboard;
use clipboard::Direction::Incoming;

use super::Peer;

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

pub fn start(channel: Sender<clipboard::Message>, host: Peer, peers: Vec<Peer>) {
	Thread::spawn(move || -> () {
		let     listener = TcpListener::bind((host.ip.as_slice(), host.port));
		let mut acceptor = listener.listen();

		for conn in acceptor.incoming() {
			if conn.is_err() {
				continue;
			}

			let mut conn = conn.unwrap();
			let     peer = peers.iter().find(|p|
				p.ip == format!("{}", conn.peer_name().unwrap().ip));

			if peer.is_none() {
				continue;
			}

			let peer    = peer.unwrap().clone();
			let host    = host.clone();
			let channel = channel.clone();

			Thread::spawn(move || -> () {
				debug!("server: connected");

				let mut ctx = SslContext::new(Sslv23).unwrap();

				ctx.set_cipher_list("DEFAULT");

				if let Some(ref cert) = host.cert {
					ctx.set_certificate_file(cert, X509FileType::PEM);
					ctx.set_private_key_file(&host.key.unwrap(), X509FileType::PEM);
				}
				else {
					let gen = X509Generator::new()
			 	 	          .set_bitlength(2048)
			 	 	          .set_valid_period(365*2)
			 	 	          .set_CN("Symbiote Inc.")
			 	 	          .set_sign_hash(SHA256)
			 	 	          .set_usage(&[DigitalSignature]);

					let (cert, key) = gen.generate().unwrap();

					let dir  = TempDir::new("symbiotic").unwrap();
					let path = dir.into_inner();

					let mut cert_path = path.clone();
					        cert_path.push("cert.pem");

					let mut file = File::open_mode(&cert_path, Open, Write).unwrap();
					cert.write_pem(&mut file).unwrap();

					let mut key_path = path.clone();
					        key_path.push("key.pem");

					let mut file = File::open_mode(&key_path, Open, Write).unwrap();
					key.write_pem(&mut file).unwrap();

					ctx.set_certificate_file(&cert_path, X509FileType::PEM);
					ctx.set_private_key_file(&key_path, X509FileType::PEM);
				}

				if let Some(ref cert) = peer.cert {
					ctx.set_verify(SslVerifyPeer, None);
					ctx.set_CA_file(cert);
				}
				else {
					ctx.set_verify(SslVerifyNone, None);
				}

				let mut conn = SslStream::new_server(&ctx, conn).unwrap();
				let mut conn = CodedInputStream::new(&mut conn);

				debug!("server: fo shizzle");

				if let Ok(msg) = parse_length_delimited_from::<protocol::handshake::Identity>(&mut conn) {
					if !verify(&msg) {
						debug!("server: handshake invalid");

						return;
					}
				}
				else {
					debug!("server: handshake error");
					return;
				}

				debug!("server: handshake verified");

				loop {
					if let Ok(mut msg) = parse_length_delimited_from::<protocol::clipboard::Change>(&mut conn) {
						debug!("server: received {:?}", msg);

						channel.send(Incoming(Arc::new((msg.get_at(),
							msg.take_content().into_iter()
								 .map(|mut c| (c.take_format(), c.take_data()))
								 .collect())))).unwrap();
					}
					else {
						break;
					}
				}
			});
		}
	});
}
