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

use std::ptr;

#[allow(non_camel_case_types)]
mod x11 {
	extern crate libc;

	use self::libc::{c_void, c_int, c_uint, c_ulong, c_long};
	use std::ops::Drop;

	pub type Id = c_ulong;

	static PROPERTY_CHANGE_MASK: c_long = 1 << 22;

	#[link(name = "X11")]
	extern "system" {
		fn XOpenDisplay(name: *const i8) -> *mut c_void;
		fn XCloseDisplay(display: *mut c_void);

		fn XDefaultRootWindow(display: *mut c_void) -> Id;

		fn XCreateSimpleWindow(display: *mut c_void, parent: Id,
		                       x: c_int, y: c_int, width: c_uint, height: c_uint,
		                       border_width: c_uint, border: Id, background: Id) -> Id;
	}

	pub struct Display {
		pointer: *mut c_void,
	}

	pub struct Window {
		id: Id,
	}

	impl Display {
		pub fn open(name: &str) -> Option<Box<Display>> {
			let pointer = unsafe { XOpenDisplay(name.to_c_str().as_ptr()) };

			if pointer.is_null() {
				None
			}
			else {
				Some(box Display { pointer: pointer })
			}
		}

		pub fn root(&self) -> Box<Window> {
			box Window { id: unsafe { XDefaultRootWindow(self.pointer) } }
		}
	}

	impl Drop for Display {
		fn drop(&mut self) {
			unsafe {
				XCloseDisplay(self.pointer);
			}
		}
	}

	impl Window {
		pub fn open(display: &Box<Display>, parent: &Box<Window>,
		            position: (i32, i32), size: (u32, u32),
		            border: (u32, Id), background: Id) -> Box<Window> {
			let (x, y)                    = position;
			let (width, height)           = size;
			let (border_width, border_id) = border;

			box Window {
				id: unsafe { XCreateSimpleWindow(display.pointer, parent.id,
			                                   x, y, width, height,
			                                   border_width, border_id, background) } }
		}
	}
}

pub struct Clipboard {
	display: Box<x11::Display>,
	window:  Box<x11::Window>,
}

impl Clipboard {
	pub fn new(specs: Option<toml::Value>) -> Clipboard {
		let mut name = "";

		if specs.is_some() {
			let table = specs.unwrap();

			if table.lookup("display").is_some() {
				name = table.lookup("display").unwrap().as_str().unwrap().clone();
			}
		}

		let display = x11::Display::open(name).unwrap();
		let window  = x11::Window::open(&display, &display.root(), (0, 0), (1, 1), (0, 0), 0);

		Clipboard { display: display, window: window }
	}

	fn get(&self) -> (&str, &[u8]) {
		("text/plain", [].as_slice())
	}

	fn set(&self, desc: (&str, &[u8])) {

	}

	fn observe(&self, func: |(&str, &[u8])| -> ()) {

	}
}
