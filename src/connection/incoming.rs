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

use std::sync::Arc;
use std::thread;
use std::sync::mpsc::Sender;

use std::fs::File;
use tempdir::TempDir;

use std::net::TcpListener;

use openssl::ssl::SslMethod::Sslv23;
use openssl::ssl::{SslContext, SslStream, SSL_VERIFY_NONE, SSL_VERIFY_PEER};
use openssl::x509::{X509FileType, X509Generator};
use openssl::x509::extension::Extension::{KeyUsage};
use openssl::x509::extension::KeyUsageOption::DigitalSignature;
use openssl::crypto::hash::Type::SHA256;

use protocol;

use protobuf::core::parse_length_delimited_from;
use protobuf::stream::{CodedInputStream};

use clipboard;
use clipboard::Direction::Incoming;

use super::Peer;

fn verify(msg: &protocol::handshake::Identity) -> bool {
	if msg.get_name() != "clipboard" {
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
	thread::spawn(move || {
		let listener = TcpListener::bind((&host.ip[..], host.port)).unwrap();

		for conn in listener.incoming() {
			if conn.is_err() {
				continue;
			}

			let conn = conn.unwrap();
			let addr = conn.peer_addr().unwrap();
			let peer = peers.iter().find(|p| p.ip == format!("{}", addr.ip()));

			if peer.is_none() {
				debug!("server: peer not found: {}", addr.ip());
				continue;
			}

			let peer    = peer.unwrap().clone();
			let host    = host.clone();
			let channel = channel.clone();

			thread::spawn(move || {
				debug!("server: connected");

				let mut ctx = SslContext::new(Sslv23).unwrap();

				ctx.set_cipher_list("DEFAULT").unwrap();

				if let Some(ref cert) = host.cert {
					ctx.set_certificate_file(cert, X509FileType::PEM).unwrap();
					ctx.set_private_key_file(&host.key.unwrap(), X509FileType::PEM).unwrap();
				}
				else {
					let gen = X509Generator::new()
			 	 		.set_bitlength(2048)
			 	 		.set_valid_period(365 * 2)
			 	 		.add_name("CN".into(), "Symbiote Inc.".into())
			 	 		.set_sign_hash(SHA256)
			 	 		.add_extension(KeyUsage(vec![DigitalSignature]));

					let (cert, key) = gen.generate().unwrap();

					let dir  = TempDir::new("symbiotic").unwrap();
					let path = dir.into_path();

					let mut cert_path = path.clone();
					        cert_path.push("cert.pem");

					{
						let mut file = File::open(&cert_path).unwrap();
						cert.write_pem(&mut file).unwrap();
					}

					let mut key_path = path.clone();
					        key_path.push("key.pem");

					{
						let mut file = File::open(&key_path).unwrap();
						key.write_pem(&mut file).unwrap();
					}

					ctx.set_certificate_file(&cert_path, X509FileType::PEM).unwrap();
					ctx.set_private_key_file(&key_path, X509FileType::PEM).unwrap();
				}

				if let Some(ref cert) = peer.cert {
					ctx.set_verify(SSL_VERIFY_PEER, None);
					ctx.set_CA_file(cert).unwrap();
				}
				else {
					ctx.set_verify(SSL_VERIFY_NONE, None);
				}

				let mut conn = SslStream::accept(&ctx, conn).unwrap();
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
