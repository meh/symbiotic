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

use std::io::{self, Read};
use std::path::Path;
use std::fs::File;
use std::sync::mpsc::Receiver;
use std::hash::{Hash, Hasher, SipHasher};
use std::collections::BTreeMap;

pub fn flush<T: Send + 'static>(channel: &Receiver<T>) -> Option<T> {
	if let Ok(v) = channel.try_recv() {
		let mut value = v;

		loop {
			if let Ok(v) = channel.try_recv() {
				value = v;
			}
			else {
				return Some(value);
			}
		}
	}
	else {
		return None;
	}
}

pub fn hash(content: &BTreeMap<String, Vec<u8>>) -> u64 {
	let mut hasher = SipHasher::new();
	let mut hash   = 0;

	for (ref key, ref value) in content {
		&(hash, key, value).hash(&mut hasher);
		hash = hasher.finish();
	}

	hash
}

macro_rules! wait {
	($body:expr) => (
		if let Ok(value) = $body {
			value
		}
		else {
			::std::thread::sleep(::std::time::Duration::from_secs(1));
			continue;
		}
	);
}

pub fn file_contents<P: AsRef<Path>>(path: P) -> io::Result<String> {
	let mut file    = try!(File::open(path));
	let mut content = String::new();

	try!(file.read_to_string(&mut content));

	Ok(content)
}
