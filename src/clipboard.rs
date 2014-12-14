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
use std::ops::Fn;

pub type Change = Arc<(String, Vec<u8>)>;

pub enum Direction<T> {
	Incoming(T),
	Outgoing(T),
}

pub type Message = Direction<Change>;

pub trait Clipboard {
	fn start(&self, channel: Sender<Message>);
	fn set(&self, value: Change);
}
