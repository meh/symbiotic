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

use std::io::timer::sleep;
use std::time::duration::Duration;

use std::hash;
use std::hash::Hash;

use clipboard;

#[allow(non_camel_case_types)]
mod x11 {
	extern crate libc;

	use self::libc::{c_void, c_int, c_uint, c_ulong, c_long};
	use std::ops::Drop;
	use std::ptr;

	pub type Id = c_ulong;

	pub static PROPERTY_CHANGE_MASK: c_long = 1 << 22;

	#[link(name = "X11")]
	extern "system" {
		fn XOpenDisplay(name: *const i8) -> *mut c_void;
		fn XCloseDisplay(display: *mut c_void);

		fn XDefaultRootWindow(display: *mut c_void) -> Id;

		fn XCreateSimpleWindow(display: *mut c_void, parent: Id,
		                       x: c_int, y: c_int, width: c_uint, height: c_uint,
		                       border_width: c_uint, border: Id, background: Id) -> Id;

		fn XSelectInput(display: *mut c_void, window: Id, mask: c_long) -> c_int;
	}

	pub struct Display {
		pointer: *mut c_void,
	}

	pub struct Window {
		id: Id,
	}

	impl Display {
		pub fn open(name: Option<&String>) -> Option<Display> {
			let pointer = match name {
				Some(name) => unsafe { XOpenDisplay(name.to_c_str().as_ptr()) },
				None       => unsafe { XOpenDisplay(ptr::null()) }
			};

			if pointer.is_null() {
				None
			}
			else {
				Some(Display { pointer: pointer })
			}
		}

		pub fn root(&self) -> Window {
			Window { id: unsafe { XDefaultRootWindow(self.pointer) } }
		}

		pub fn select(&self, window: &Window, mask: c_long) {
			unsafe {
				XSelectInput(self.pointer, window.id, mask);
			}
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
		pub fn open(display: &Display, parent: &Window,
		            position: (i32, i32), size: (u32, u32),
		            border: (u32, Id), background: Id) -> Window {
			let (x, y)                    = position;
			let (width, height)           = size;
			let (border_width, border_id) = border;

			Window {
				id: unsafe { XCreateSimpleWindow(display.pointer, parent.id,
				                                 x, y, width, height,
				                                 border_width, border_id, background) } }
		}
	}
}

pub struct Clipboard {
	display: Option<String>,
	channel: Option<Sender<clipboard::Change>>,
}

impl Clipboard {
	pub fn new(specs: Option<toml::Value>) -> Clipboard {
		let mut name = None;

		if specs.is_some() {
			let table = specs.unwrap();

			if table.lookup("display").is_some() {
				name = Some(table.lookup("display").unwrap().as_str().unwrap().to_string());
			}
		}

		Clipboard { display: name, channel: None }
	}
}

impl clipboard::Clipboard for Clipboard {
	fn start(&mut self, function: |clipboard::Change| -> ()) {
		let display = x11::Display::open(self.display.as_ref()).unwrap();
		let window  = x11::Window::open(&display, &display.root(), (0, 0), (1, 1), (0, 0), 0);

		display.select(&window, x11::PROPERTY_CHANGE_MASK);

		let (sender, receiver): (Sender<clipboard::Change>, Receiver<clipboard::Change>) = channel();

		spawn(proc() {
			loop {
				sleep(Duration::seconds(1));

				println!("hue");
			}
		});

		self.channel = Some(sender);
	}

	fn set(&mut self, value: clipboard::Change) {
		self.channel.as_ref().unwrap().send(value.clone());
	}
}
