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

use std::io::TcpListener;
use std::io::TcpStream;

pub struct Manager {
	port:  u16,
	hosts: Vec<String>,
}

impl Manager {
	pub fn new(port: u16, hosts: Vec<String>) -> Manager {
		Manager { port: port, hosts: hosts }
	}

	pub fn start(&self) {
		println!("{} {}", self.port, self.hosts);
	}
}
