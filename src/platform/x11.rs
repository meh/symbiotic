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

use std::sync::mpsc::Sender;

use clipboard;

pub fn start(channel: Sender<clipboard::Message>, specs: Option<toml::Value>) -> Sender<clipboard::Change> {
	let mut display = None;
	let mut source  = "PRIMARY".to_string();

	if let Some(table) = specs {
		if let Some(value) = table.lookup("display") {
			display = Some(value.as_str().unwrap().to_string());
		}

		if let Some(value) = table.lookup("mode") {
			source = match value.as_str().unwrap() {
				"selection" => "PRIMARY".to_string(),
				"clipboard" => "CLIPBOARD".to_string(),
				value       => panic!("unknown source type: {}", value),
			};
		}
	}

	lib::Manager::start(channel, display, source)
}

mod lib {
	extern crate regex;

	use self::regex::Regex;

	use std::thread::Thread;
	use std::sync::Arc;
	use std::sync::mpsc::{Sender, Receiver, channel};

	use std::hash::{self, Hash, SipHasher};

	use std::io::timer::sleep;
	use std::time::duration::Duration;

	use std::collections::BTreeMap;

	use clipboard;
	use clipboard::Direction::Outgoing;

	pub struct Manager {
		display: x::Display,
		source:  String,
		window:  x::Window,

		channel: Sender<clipboard::Message>,
	}

	#[allow(non_snake_case)]
	impl Manager {
		pub fn start(main: Sender<clipboard::Message>, display: Option<String>, source: String) -> Sender<clipboard::Change> {
			let display            = x::Display::open(display.as_ref()).unwrap();
			let (sender, receiver) = channel::<clipboard::Change>();

			Thread::spawn(move || -> () {
				let mut manager = Manager::new(main, display, source);

				loop {
					if let None = manager.serve(&receiver) {
						if let Some(change) = manager.poll() {
							manager.send(&change);
						}

						// this could be reduced a bit, but unsure about it
						sleep(Duration::seconds(1));
					}
				}
			});

			sender
		}

		pub fn new(main: Sender<clipboard::Message>, display: x::Display, source: String) -> Manager {
			let window = x::Window::open(&display, &display.root(), (0, 0), (1, 1), (0, 0), 0);

			window.select(x::PropertyChangeMask);

			Manager {
				channel: main,
				display: display,
				source:  source,
				window:  window,
			}
		}

		pub fn serve(&self, receiver: &Receiver<clipboard::Change>) -> Option<clipboard::Change> {
			if let Some(c) = self.flush(receiver) {
				let mut change = c;

				self.window.selection_owner(self.intern(self.source.as_slice()), x::CurrentTime);

				loop {
					let event = self.display.next_event();

					if let Some(..) = event.details::<x::SelectionClear>() {
						return self.flush(receiver);
					}

					if let Some(details) = event.details::<x::SelectionRequest>() {
						if let Some(c) = self.flush(receiver) {
							change = c;
						}

						let name    = self.name(details.target);
						let content = change.1.clone().into_iter().collect::<BTreeMap<String, Vec<u8>>>();

						match name.as_slice() {
							"TARGETS" => {
								let mut targets: Vec<x::Atom> =
									content.keys().map(|mime| self.intern(mime.as_slice())).collect();

								if content.contains_key("text/plain") {
									targets.push(self.intern("UTF8_STRING"));
								}

								self.set(details, &targets);
							},

							"UTF8_STRING" | "STRING" => {
								if let Some(value) = content.get("text/plain") {
									self.set(details, value);
								}
								else {
									self.error(details);
								}
							},

							name if Regex::new(r"^(.*?)/(.*?)$").unwrap().is_match(name) => {
								if let Some(value) = content.get(name) {
									self.set(details, value);
								}
								else {
									self.error(details);
								}
							},

							_ => {
								self.error(details);
							}
						}
					}
				}
			}

			None
		}

		fn flush(&self, receiver: &Receiver<clipboard::Change>) -> Option<clipboard::Change> {
			if let Ok(c) = receiver.try_recv() {
				let mut change = c;

				loop {
					if let Ok(c) = receiver.try_recv() {
						change = c;
					}
					else {
						return Some(change);
					}
				}
			}
			else {
				return None;
			}
		}

		fn set<T>(&self, details: &x::SelectionRequest, data: &Vec<T>) {
			let window = self.display.window(details.requestor);
			let kind   = match self.name(details.target).as_slice() {
				"TARGETS" => self.intern("ATOM"),
				name      => self.intern(name)
			};

			let mut event = self.display.event();
			{
				let notify = event.mut_details::<x::SelectionNotify>();

				notify.requestor = details.requestor;
				notify.selection = self.intern(self.source.as_slice());
				notify.target    = details.target;
				notify.property  = details.property;
				notify.time      = details.time;
			}

			window.set_property(details.property, kind, data);
			window.send(&event);
		}

		fn error(&self, details: &x::SelectionRequest) {
			let     window = self.display.window(details.requestor);
			let mut event  = self.display.event();
			{
				let notify = event.mut_details::<x::SelectionNotify>();

				notify.requestor = details.requestor;
				notify.selection = self.intern(self.source.as_slice());
				notify.target    = details.target;
				notify.property  = 0;
				notify.time      = details.time;
			}

			window.send(&event);
		}

		pub fn poll(&mut self) -> Option<clipboard::Change> {
			static mut hash:      u64 = 0;
			static mut timestamp: u64 = 0;

			let mut current: u64 = 0;
			let mut content: BTreeMap<String, Vec<u8>> = BTreeMap::new();
			
			if let Some(property) = self.get("UTF8_STRING") {
				content.insert("text/plain".to_string(),
					property.items::<u8>().unwrap().to_vec());
			}
			else if let Some(property) = self.get("STRING") {
				content.insert("text/plain".to_string(),
					property.items::<u8>().unwrap().to_vec());
			}

			if let Some(property) = self.get("TARGETS") {
				for atom in property.items::<x::Atom>().unwrap().iter() {
					let name = self.name(*atom);

					if Regex::new(r"^(.*?)/(.*?)$").unwrap().is_match(name.as_slice()) {
						if let Some(value) = self.get(name.as_slice()) {
							content.insert(name.clone(),
								value.items::<u8>().unwrap().to_vec());
						}
					}
					else {
						if let "TIMESTAMP" = name.as_slice() {
							current = self.get("TIMESTAMP").unwrap().items::<u32>().unwrap()[0] as u64;
						}
					}
				}
			}

			if content.len() == 0 {
				return None;
			}

			unsafe {
				if current != 0 {
					hash = 0;

					if current == timestamp {
						return None;
					}

					timestamp = current;
				}
				else {
					timestamp = 0;

					for (ref key, ref value) in content.iter() {
						current = hash::hash::<_, SipHasher>(&(current, key, value));
					}

					if current == hash {
						return None;
					}

					hash = current;
				}
			}

			Some(Arc::new((unsafe { timestamp }, content.into_iter().collect())))
		}

		fn intern(&self, name: &str) -> x::Atom {
			self.display.intern_atom(name)
		}

		fn name(&self, atom: x::Atom) -> String {
			self.display.atom_name(atom)
		}

		fn get(&self, name: &str) -> Option<x::Property> {
			let id = self.intern("SYMBIOTIC");

			self.window.convert_selection(
				self.intern(self.source.as_slice()),
				self.intern(name),
				id);

			loop {
				let event = self.display.next_event();

				if let Some(details) = event.details::<x::SelectionNotify>() {
					if details.property == 0 {
						return None;
					}

					let property = self.window.get_property(id);

					self.window.delete_property(id);

					return property;
				}
			}
		}

		pub fn send(&self, change: &clipboard::Change) {
			self.channel.send(Outgoing(change.clone())).unwrap();
		}
	}

	#[allow(non_camel_case_types, non_upper_case_globals)]
	mod x {
		extern crate libc;

		use self::libc::{c_void, c_int, c_uint, c_ulong, c_long, c_uchar};

		use std::ops::Drop;

		use std::ptr;
		use std::mem;
		use std::intrinsics::type_id;
		use std::slice;

		use std::ffi::c_str_to_bytes;
		use std::ffi::CString;

		use std::collections::BTreeMap;

		pub type Id     = c_ulong;
		pub type Atom   = c_ulong;
		pub type Time   = c_ulong;
		pub type Bool   = c_int;
		pub type Status = c_int;

		pub static PropertyChangeMask: c_long = 1 << 22;

		pub static False: Bool = 0;
		pub static True: Bool  = 1;

		pub static CurrentTime: Time = 0;

		pub static PropModeReplace: c_int = 0;

		#[derive(Show)]
		#[repr(C)]
		pub struct Event {
			pub kind:    c_int,
			pub serial:  c_ulong,
			pub fake:    c_int,
			pub display: *const c_void,

			_data: [u64; 23us]
		}

		#[derive(Show)]
		#[repr(C)]
		pub struct SelectionNotify {
			pub requestor: Id,
			pub selection: Atom,
			pub target:    Atom,
			pub property:  Atom,
			pub time:      Time,
		}

		#[derive(Show)]
		#[repr(C)]
		pub struct SelectionClear {
			pub window:    Id,
			pub selection: Atom,
			pub time:      Time,
		}

		#[derive(Show)]
		#[repr(C)]
		pub struct SelectionRequest {
			pub owner:     Id,
			pub requestor: Id,
			pub selection: Atom,
			pub target:    Atom,
			pub property:  Atom,
			pub time:      Time,
		}

		impl Event {
			pub fn new() -> Event {
				unsafe {
					mem::zeroed()
				}
			}

			pub fn details<T>(&self) -> Option<&mut T> where T: 'static {
				unsafe {
					if (self.kind == 31 && type_id::<T>() == type_id::<SelectionNotify>()) ||
					   (self.kind == 30 && type_id::<T>() == type_id::<SelectionRequest>()) ||
					   (self.kind == 29 && type_id::<T>() == type_id::<SelectionClear>())
					{
						Some(mem::transmute(&self._data))
					}
					else {
						None
					}
				}
			}

			pub fn mut_details<T>(&mut self) -> &mut T where T: 'static {
				unsafe {
					if type_id::<T>() == type_id::<SelectionNotify>() {
						self.kind = 31;
					}

					mem::transmute(&self._data)
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

			fn XSendEvent(display: *const c_void, window: Id, propagate: Bool,
			              mask: c_long, event: *const Event) -> Status;

			fn XConvertSelection(display: *const c_void, selection: Atom, target: Atom, property: Atom,
			                     requestor: Id, time: Time) -> c_int;

			fn XGetWindowProperty(display: *const c_void, w: Id, property: Atom,
			                      long_offset: c_long, long_length: c_long,
			                      delete: Bool, req_type: Atom,
			                      actual_type_return: *mut Atom, actual_format_return: *mut c_int,
			                      nitems_return: *mut c_ulong, bytes_after_return: *mut c_ulong,
			                      prop_return: *mut *mut c_uchar) -> c_int;

			fn XChangeProperty(display: *const c_void, window: Id, property: Atom,
			                   kind: Atom, format: c_int, mode: c_int,
			                   data: *const c_void, length: c_int) -> c_int;

			fn XDeleteProperty(display: *const c_void, window: Id, property: Atom) -> c_int;

			fn XMaxRequestSize(display: *const c_void) -> c_long;

			fn XSetSelectionOwner(display: *const c_void, selection: Atom, owner: Id, time: Time) -> c_int;

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

		pub struct Display {
			pointer: *const c_void,
		}

		pub struct Window {
			display: *const c_void,
			id:      Id,
		}

		#[derive(Show)]
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
						Some(slice::from_raw_buf(mem::transmute(&self.data.as_ptr()), self.items as usize))
					}
				}

				if self.format == 32 && mem::size_of::<T>() == 8 {
					return unsafe {
						Some(slice::from_raw_buf(mem::transmute(&self.data.as_ptr()), (self.items / 2) as usize))
					}
				}

				None
			}
		}

		impl Display {
			pub fn open(name: Option<&String>) -> Option<Display> {
				let pointer = if let Some(name) = name {
					unsafe { XOpenDisplay(CString::from_slice(name.as_bytes()).as_ptr()) }
				}
				else {
					unsafe { XOpenDisplay(ptr::null()) }
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
					id:      unsafe { XDefaultRootWindow(self.pointer) }
				}
			}

			pub fn window(&self, id: Id) -> Window {
				Window {
					display: self.pointer,
					id:      id,
				}
			}

			pub fn next_event(&self) -> Event {
				unsafe {
					let mut event: Event = mem::uninitialized();

					XNextEvent(self.pointer, (&mut event) as *mut _);

					event
				}
			}

			pub fn event(&self) -> Event {
				let mut event = Event::new();

				event.display = self.pointer;

				return event;
			}

			pub fn intern_atom(&self, name: &str) -> Atom {
				unsafe {
					match name {
						"PRIMARY" => {
							1
						},

						"SECONDARY" => {
							2
						},

						"ATOM" => {
							4
						},

						"STRING" => {
							31
						},

						"UTF8_STRING" => {
							XmuInternAtom(self.pointer, _XA_UTF8_STRING)
						},

						"CLIPBOARD" => {
							XmuInternAtom(self.pointer, _XA_CLIPBOARD)
						},

						"TIMESTAMP" => {
							XmuInternAtom(self.pointer, _XA_TIMESTAMP)
						},

						"TARGETS" => {
							XmuInternAtom(self.pointer, _XA_TARGETS)
						},

						_ => {
							XInternAtom(self.pointer, CString::from_slice(name.as_bytes()).as_ptr(), False)
						}
					}
				}
			}

			pub fn atom_name(&self, id: Atom) -> String {
				if id == 0 {
					return "None".to_string();
				}

				unsafe {
					let buffer = XGetAtomName(self.pointer, id);
					let result = String::from_utf8_lossy(c_str_to_bytes(&buffer)).into_owned();

					XFree(buffer as *mut c_void);

					result
				}
			}
		}

		unsafe impl Send for Display { }

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

			pub fn send(&self, event: &Event) -> Status {
				unsafe {
					XSendEvent(self.display, self.id, 0, 0, event)
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

					XGetWindowProperty(self.display, self.id, id, 0, XMaxRequestSize(self.display), False, 0, &mut kind, &mut format, &mut items, &mut after, &mut buffer);

					if kind == 0 {
						return None;
					}

					data = Vec::from_raw_buf(buffer as *const u8, (items * ((format / 8) as u64)) as usize);

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

			pub fn set_property<T>(&self, id: Atom, kind: Atom, data: &Vec<T>) {
				let (format, length) = match mem::size_of::<T>() * 8 {
					8  => (8,  data.len()),
					16 => (16, data.len()),
					32 => (32, data.len()),
					64 => (32, data.len() * 2),

					s => panic!("unsupported size: {:?}", s)
				};

				unsafe {
					XChangeProperty(self.display, self.id, id, kind, format, PropModeReplace,
					                data.as_ptr() as *const c_void,
					                length as c_int);
				}
			}
			
			pub fn delete_property(&self, id: Atom) {
				unsafe {
					XDeleteProperty(self.display, self.id, id);
				}
			}

			pub fn select(&self, mask: c_long) {
				unsafe {
					XSelectInput(self.display, self.id, mask);
				}
			}

			pub fn selection_owner(&self, selection: Atom, time: Time) {
				unsafe {
					XSetSelectionOwner(self.display, selection, self.id, time);
				}
			}
		}
	}
}
