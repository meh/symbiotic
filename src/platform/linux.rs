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

use std::thread::Thread;
use std::sync::Arc;

use clipboard;
use clipboard::{Message, Change};
use clipboard::Direction::Outgoing;

#[allow(non_camel_case_types, non_upper_case_globals)]
mod x11 {
	extern crate libc;

	use self::libc::{c_void, c_int, c_uint, c_ulong, c_long, c_uchar};
	use std::ops::Drop;
	use std::ptr;
	use std::mem;
	use std::c_str::CString;
	use std::slice;
	use std::collections::BTreeMap;

	pub type Id   = c_ulong;
	pub type Atom = c_ulong;
	pub type Time = c_ulong;
	pub type Bool = c_int;

	pub static PropertyChangeMask: c_long = 1 << 22;

	pub static False: Bool = 0;
	pub static True: Bool  = 1;

	pub static CurrentTime: Time = 0;

	#[deriving(Show)]
	#[repr(C)]
	pub struct Event {
		pub kind:    c_int,
		pub serial:  c_ulong,
		pub fake:    c_int,
		pub display: *const c_void,
		pub window:  Id,

		_data: [u64, ..20u]
	}

	#[deriving(Show)]
	#[repr(C)]
	pub struct SelectionNotify {
		pub selection: Atom,
		pub target:    Atom,
		pub property:  Atom,
		pub time:      Time,
	}

	impl Event {
		pub fn as_selection_notify(&self) -> Option<&SelectionNotify> {
			if self.kind == 31 {
				Some(unsafe {
					::std::mem::transmute(&self._data)
				})
			}
			else {
				None
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

		fn XGetAtomName(display: *const c_void, id: Atom) -> *const i8;

		fn XNextEvent(display: *const c_void, event: *mut Event) -> c_int;

		fn XConvertSelection(display: *const c_void, selection: Atom, target: Atom, property: Atom,
		                     requestor: Id, time: Time) -> c_int;

		fn XGetWindowProperty(display: *const c_void, w: Id, property: Atom,
		                      long_offset: c_long, long_length: c_long,
		                      delete: Bool, req_type: Atom,
		                      actual_type_return: *mut Atom, actual_format_return: *mut c_int,
		                      nitems_return: *mut c_ulong, bytes_after_return: *mut c_ulong,
		                      prop_return: *mut *mut c_uchar) -> c_int;

		fn XDeleteProperty(display: *const c_void, window: Id, property: Atom) -> c_int;

		fn XFetchBuffer(display: *const c_void, length: *mut c_int, buffer: c_int) -> *mut c_uchar;

		fn XFree(data: *mut c_void) -> c_int;
	}

	#[link(name = "Xmu")]
	extern "system" {
		static _XA_CLIPBOARD:   *const c_void;
		static _XA_UTF8_STRING: *const c_void;
		static _XA_TARGETS:     *const c_void;
		static _XA_TIMESTAMP:   *const c_void;

		fn XmuInternAtom(display: *const c_void, atom: *const c_void) -> Atom;
	}

	pub struct AtomCache {
		display: *const c_void,
		map:     BTreeMap<String, Atom>,
	}

	impl AtomCache {
		pub fn new(display: *const c_void) -> AtomCache {
			AtomCache {
				display: display,
				map:     BTreeMap::new(),
			}
		}

		pub fn intern(&mut self, name: &str) -> Atom {
			if let Some(id) = self.map.get(&name.to_string()) {
				return *id;
			}

			let id = unsafe {
				match name {
					"PRIMARY" => {
						1
					},

					"SECONDARY" => {
						2
					},

					"STRING" => {
						31
					},

					"UTF8_STRING" => {
						XmuInternAtom(self.display, _XA_UTF8_STRING)
					},

					"CLIPBOARD" => {
						XmuInternAtom(self.display, _XA_CLIPBOARD)
					},

					"TIMESTAMP" => {
						XmuInternAtom(self.display, _XA_TIMESTAMP)
					},

					"TARGETS" => {
						XmuInternAtom(self.display, _XA_TARGETS)
					},

					_ => {
						XInternAtom(self.display, name.to_c_str().as_ptr(), False)
					}
				}
			};

			self.map.insert(name.to_string(), id);

			id
		}

		pub fn name(&self, id: Atom) -> String {
			if id == 0 {
				return "None".to_string();
			}

			unsafe {
				let buffer = XGetAtomName(self.display, id);
				let result = String::from_str(CString::new(buffer, false).as_str().unwrap());

				XFree(buffer as *mut c_void);

				result
			}
		}

		pub fn hint(&mut self, name: &str, id: Atom) {
			self.map.insert(name.to_string(), id);
		}
	}

	pub struct Display {
		pointer: *const c_void,
	}

	pub struct Window {
		display: *const c_void,
		id:      Id,
	}

	#[deriving(Show)]
	pub struct Property<'a> {
		pub id:   Atom,
		pub kind: Atom,

		format: c_int,
		items:  c_ulong,
		data:   Vec<u8>,
	}

	impl<'a> Property<'a> {
		pub fn items<T>(&self) -> Option<&'a [T]> {
			if self.format == (mem::size_of::<T>() * 8) as i32 {
				return unsafe {
					Some(slice::from_raw_buf(mem::transmute(&self.data.as_ptr()), self.items as uint))
				}
			}

			if self.format == 32 && mem::size_of::<T>() == 8 {
				return unsafe {
					Some(slice::from_raw_buf(mem::transmute(&self.data.as_ptr()), (self.items / 2) as uint))
				}
			}

			None
		}
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

		pub fn atom(&self) -> AtomCache {
			AtomCache::new(self.pointer)
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

		pub fn next_event(&self) -> Event {
			unsafe {
				let mut event: Event = mem::uninitialized();

				XNextEvent(self.pointer, (&mut event) as *mut _);

				event
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

		pub fn get_property(&self, id: Atom) -> Option<Property> {
			unsafe {
				let mut buffer: *mut c_uchar = mem::uninitialized();
				let mut kind:   Atom         = mem::uninitialized();
				let mut format: c_int        = mem::uninitialized();
				let mut items:  c_ulong      = mem::uninitialized();
				let mut after:  c_ulong      = mem::uninitialized();
				let     data:   Vec<u8>;

				XGetWindowProperty(self.display, self.id, id, 0, !0, False, 0, &mut kind, &mut format, &mut items, &mut after, &mut buffer);

				if kind == 0 {
					return None;
				}

				data = Vec::from_raw_buf(buffer as *const u8, (items * ((format / 8) as u64)) as uint);

				XFree(buffer as *mut c_void);

				Some(Property {
					id:     id,
					kind:   kind,
					format: format,
					items:  items,
					data:   data
				})
			}
		}
		
		pub fn delete_property(&self, id: Atom) {
			unsafe {
				XDeleteProperty(self.display, self.id, id);
			}
		}
	}
}

enum Source {
	Selection,
	Clipboard,
}

pub struct Clipboard {
	display: Option<String>,
	source:  Source,
}

impl Clipboard {
	pub fn new(specs: Option<toml::Value>) -> Clipboard {
		let mut display = None;
		let mut source  = Source::Selection;

		if let Some(table) = specs {
			if let Some(value) = table.lookup("display") {
				display = Some(value.as_str().unwrap().to_string());
			}

			if let Some(value) = table.lookup("source") {
				source = match value.as_slice() {
					"selection" => Source::Selection,
					"clipboard" => Source::Clipboard,
				};
			}
		}

		Clipboard {
			display: display,
			source:  source,
		}
	}
}

#[allow(non_snake_case)]
impl clipboard::Clipboard for Clipboard {
	fn start(&self, ipc: Sender<Message>) {
		let config  = self.clone();
		let display = x11::Display::open(config.display.as_ref()).unwrap();
		let window  = x11::Window::open(&display, &display.root(), (0, 0), (1, 1), (0, 0), 0);
		let source  = match config.source {
			Source::Selection => display.intern("PRIMARY"),
			Source::Clipboard => display.intern("CLIPBOARD"),
		};

		display.select(&window, x11::PropertyChangeMask);

		Thread::spawn(move || -> () {
			#[deriving(Show)]
			enum State {
				None,
				Formats,
				Timestamp,
				Fetch,
				Done,
			}

			struct Channel {
				channel: Sender<Message>,
				hash:    u64,
				time:    u64,
				id:      u64,
			}
			
			impl Channel {
				fn new(channel: Sender<Message>) -> Channel {
					Channel {
						channel: channel,
						hash:    0,
						id:      0,
						time:    0,
					}
				}

				// TODO: actually implement it
				fn next(&mut self, timestamp: u64, format: &str, data: &Vec<u8>) -> u64 {
					0
				}

				fn send(&mut self, timestamp: u64, format: &str, data: Vec<u8>) {
					let id      = self.next(timestamp, format, &data);
					let message = (timestamp, format.to_string(), data);

					self.channel.send(Outgoing(Arc::new(message)));
				}
			}

			let mut atom      = display.atom();
			let mut channel   = Channel::new(ipc);
			let mut state     = State::None;
			let mut formats   = Vec::new();
			let mut timestamp = 0u64;

			loop {
				debug!("STATE: {}", state);

				match state {
					State::None => {
						window.convert_selection(source, atom.intern("TARGETS"), atom.intern("SYMBIOTIC"));

						state = State::Formats;
					},

					State::Formats => {
						let event = display.next_event();

						if let Some(details) = event.as_selection_notify() {
							if details.property != 0 {
								if let Some(property) = window.get_property(atom.intern("SYMBIOTIC")) {
									for id in property.items::<x11::Atom>().unwrap().iter() {
										let name = atom.name(*id);

										atom.hint(name.as_slice(), *id);
										formats.push(name);
									}

									window.delete_property(atom.intern("SYMBIOTIC"));

									debug!("formats: {}", formats);
								}
							}

							state = State::Done;
						}
					},

					State::Timestamp => {
					},

					State::Fetch => {

					},

					State::Done => {
						timestamp = 0;
						state     = State::None;
						formats   = Vec::new();

						sleep(Duration::seconds(1));
					}
				}
			}
		}).detach();
	}

	fn set(&self, value: Change) {
	}
}
