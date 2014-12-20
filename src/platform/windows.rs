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

extern crate toml;

use std::sync::Arc;

use clipboard;
use clipboard::{Message, Change};
use clipboard::Direction::Outgoing;

#[deriving(Clone, Show)]
pub struct Clipboard {
	x: int,
}

impl Clipboard {
	pub fn new(specs: Option<toml::Value>) -> Clipboard {
		Clipboard {
			x: 42
		}
	}
}

impl clipboard::Clipboard for Clipboard {
	fn start(&self, ipc: Sender<Message>) {
	}

	fn set(&self, value: Change) {
	}
}
