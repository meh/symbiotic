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

use std::sync::Arc;

use clipboard;
use clipboard::Change;

#[allow(non_camel_case_types, non_upper_case_globals)]
mod x11 {
	extern crate libc;

	use self::libc::{c_void, c_int, c_uint, c_ulong, c_long, c_uchar};
	use std::ops::Drop;
	use std::ptr;

	pub type Id   = c_ulong;
	pub type Atom = c_ulong;
	pub type Time = c_ulong;
	pub type Bool = c_int;

	pub static PROPERTY_CHANGE_MASK: c_long = 1 << 22;
	pub static False: Bool = 0;
	pub static True: Bool  = 1;

	pub static CurrentTime: Time = 0;

	#[allow(dead_code)]
	pub mod event {
		extern crate libc;

		use self::libc::{c_void, c_int, c_uint, c_ulong, c_long};

		#[repr(C)]
		pub struct Event {
			kind:    c_int,
			serial:  c_ulong,
			fake:    c_int,
			display: *const c_void,
			window:  super::Id,

			_data: [u64, ..20u]
		}

		#[repr(C)]
		pub struct Selection {
			selection: super::Atom,
			target:    super::Atom,
			property:  super::Atom,
			time:      super::Time,
		}

		impl Event {
			pub fn details<T>(&mut self) -> *mut T {
				unsafe {
					::std::mem::transmute(&self._data)
				}
			}
		}
	}

	#[link(name = "X11")]
	extern "system" {
		fn XOpenDisplay(name: *const i8) -> *const c_void;

		fn XCloseDisplay(display: *const c_void);

		fn XDefaultRootWindow(display: *const c_void) -> Id;

		fn XCreateSimpleWindow(display: *const c_void, parent: Id,
		                       x: c_int, y: c_int, width: c_uint, height: c_uint,
		                       border_width: c_uint, border: Id, background: Id) -> Id;

		fn XSelectInput(display: *const c_void, window: Id, mask: c_long) -> c_int;

		fn XInternAtom(display: *const c_void, name: *const i8, if_exist: c_int) -> Atom;

		fn XNextEvent(display: *const c_void, event: *mut c_void) -> c_int;

		fn XConvertSelection(display: *const c_void, selection: Atom, target: Atom, property: Atom,
		                     requestor: Id, time: Time) -> c_int;

		fn XFetchBuffer(display: *const c_void, length: *mut c_int, buffer: c_int) -> *mut c_uchar;

		fn XFree(data: *mut c_void) -> c_int;
	}

	pub struct Display {
		pointer: *const c_void,
	}

	pub struct Window {
		display: *const c_void,
		id:      Id,
	}

	impl Display {
		pub fn open(name: Option<&String>) -> Option<Display> {
			let pointer = match name {
				Some(name) => unsafe {
					XOpenDisplay(name.to_c_str().as_ptr())
				},

				None => unsafe {
					XOpenDisplay(ptr::null())
				}
			};

			if pointer.is_null() {
				None
			}
			else {
				Some(Display { pointer: pointer })
			}
		}

		pub fn root(&self) -> Window {
			Window {
				display: self.pointer,

				id: unsafe {
					XDefaultRootWindow(self.pointer)
				}
			}
		}

		pub fn select(&self, window: &Window, mask: c_long) {
			unsafe {
				XSelectInput(self.pointer, window.id, mask);
			}
		}
		
		pub fn intern(&self, name: &str) -> Atom {
			unsafe {
				XInternAtom(self.pointer, name.to_c_str().as_ptr(), False)
			}
		}

		pub fn fetch_buffer(&self, id: i32) -> Vec<u8> {
			unsafe {
				let mut length: c_int = 0;
				let     buffer: *mut c_uchar;
				let     result: Vec<u8>;
				
				buffer = XFetchBuffer(self.pointer, &mut length, id);
				result = Vec::from_raw_buf(buffer as *const u8, length as uint);

				XFree(buffer as *mut c_void);

				result
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
		            (x, y): (i32, i32), (width, height): (u32, u32),
		            (border_width, border_id): (u32, Id), background: Id) -> Window {
			Window {
				display: display.pointer,

				id: unsafe {
					XCreateSimpleWindow(display.pointer, parent.id,
					                    x, y, width, height,
					                    border_width, border_id, background)
				}
			}
		}

		pub fn convert_selection(&self, selection: Atom, target: Atom, property: Atom) -> i32 {
			unsafe {
				XConvertSelection(self.display, selection, target, property, self.id, CurrentTime)
			}
		}
	}
}

#[deriving(Clone, Show)]
enum Selection {
	Primary,
	Secondary,
	Clipboard,
	String,
}

#[deriving(Clone, Show)]
struct Config {
	display:   Option<String>,
	selection: Selection,
}

pub struct Clipboard {
	config:  Config,
	channel: Option<Sender<Change>>,
}

impl Clipboard {
	pub fn new(specs: Option<toml::Value>) -> Clipboard {
		let mut display   = None;
		let mut selection = Selection::Primary;

		if specs.is_some() {
			let table = specs.unwrap();

			if table.lookup("display").is_some() {
				display = Some(table.lookup("display").unwrap().as_str().unwrap().to_string());
			}

			if table.lookup("selection").is_some() {
				selection = match table.lookup("selection").unwrap().as_str().unwrap() {
					"primary"   => Selection::Primary,
					"secondary" => Selection::Secondary,
					"clipboard" => Selection::Clipboard,
					"string"    => Selection::String,

					_ => panic!("unknown selection type")
				}
			}
		}

		Clipboard {
			channel: None,

			config: Config {
				display:   display,
				selection: selection,
			}
		}
	}
}

impl clipboard::Clipboard for Clipboard {
	fn start<F>(&mut self, function: F) where F: Fn(Change) + Send {
		let config    = self.config.clone();
		let display   = x11::Display::open(config.display.as_ref()).unwrap();
		let window    = x11::Window::open(&display, &display.root(), (0, 0), (1, 1), (0, 0), 0);

		display.select(&window, x11::PROPERTY_CHANGE_MASK);

		let (sender, receiver): (Sender<Change>, Receiver<Change>) = channel();

		spawn(proc() {
			enum State {
				None,
				Sent,
				Incr,
				Fallback,
				Done,
			}

			let property = display.intern("SYMBIOTIC_CLIPBOARD");

			let mut format: x11::Atom;
			let mut state = State::None;

			loop {
				match state {
					State::None => {
						match config.selection {
							Selection::String => {
								function.call((Arc::new(("text/plain".to_string(), display.fetch_buffer(0))),));

								state = State::Done;
							},

							_ => {
								/*window.convert_selection()*/
							}
						}
					},

					State::Sent => {

					},

					State::Incr => {

					},

					State::Fallback => {

					},

					State::Done => {
						state = State::None;

						sleep(Duration::seconds(1));
					}
				}
			}
		});

		self.channel = Some(sender);
	}

	fn set(&mut self, value: Change) {
		self.channel.as_ref().unwrap().send(value.clone());
	}
}
