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

#[derive(Eq, PartialEq, Copy)]
pub enum Mode {
	Selection,
	Clipboard,
}

pub fn start(channel: Sender<clipboard::Message>, specs: Option<toml::Value>) -> Sender<clipboard::Change> {
	let mut display = None;
	let mut mode    = Mode::Selection;

	if let Some(table) = specs {
		if let Some(value) = table.lookup("display") {
			display = Some(value.as_str().unwrap().to_string());
		}

		if let Some(value) = table.lookup("mode") {
			mode = match value.as_str().unwrap() {
				"selection" => Mode::Selection,
				"clipboard" => Mode::Clipboard,
				value       => panic!("unknown source type: {}", value),
			};
		}
	}

	lib::Manager::start(channel, display, mode)
}

mod lib {
	extern crate regex;

	use std::thread::Thread;
	use std::sync::Arc;
	use std::sync::mpsc::{Sender, Receiver, channel};
	use std::cell::Cell;

	use std::hash::{self, SipHasher};

	use std::old_io::timer::sleep;
	use std::time::duration::Duration;

	use std::collections::BTreeMap;
	use std::default::Default;

	use utils;

	use clipboard;
	use clipboard::Direction::Outgoing;
	use super::Mode;

	pub struct Manager {
		display: x::Display,
		window:  x::Window,

		mode:      Mode,
		primary:   State,
		clipboard: State,

		channel: Sender<clipboard::Message>,
	}

	struct State {
		hash:      Cell<u64>,
		timestamp: Cell<u64>,
	}

	impl Default for State {
		fn default() -> Self {
			State {
				hash:      Cell::new(0),
				timestamp: Cell::new(0),
			}
		}
	}

	#[allow(non_snake_case)]
	impl Manager {
		pub fn start(main: Sender<clipboard::Message>, display: Option<String>, mode: Mode) -> Sender<clipboard::Change> {
			let (sender, receiver) = channel::<clipboard::Change>();

			Thread::spawn(move || -> () {
				let display = x::Display::open(display.as_ref()).unwrap();
				let manager = Manager::new(main, display, mode);

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

		pub fn new(main: Sender<clipboard::Message>, display: x::Display, mode: Mode) -> Manager {
			let window = x::Window::open(&display, &display.root(), (0, 0), (1, 1), (0, 0), 0);

			window.select(x::PropertyChangeMask);

			Manager {
				display: display,
				window:  window,

				mode:      mode,
				primary:   Default::default(),
				clipboard: Default::default(),

				channel: main,
			}
		}

		fn intern(&self, name: &str) -> x::Atom {
			self.display.intern(name)
		}

		fn atom(&self, atom: x::Atom) -> String {
			self.display.atom(atom)
		}

		pub fn serve(&self, receiver: &Receiver<clipboard::Change>) -> Option<clipboard::Change> {
			let mut change;

			if let Some(c) = utils::flush(receiver) {
				let clipboard = self.intern("CLIPBOARD");

				self.window.own_selection(match self.mode {
					Mode::Selection => self.intern("PRIMARY"),
					Mode::Clipboard => self.intern("CLIPBOARD")
				});

				if self.mode == Mode::Selection {
					self.window.own_selection(self.intern("CLIPBOARD"))
				}

				change = c;

				loop {
					let event = self.display.next_event();

					if let Some(details) = event.details::<x::SelectionClear>() {
						if self.mode == Mode::Selection {
							if details.selection != clipboard {
								self.window.disown_selection(self.intern("CLIPBOARD"));

								return utils::flush(receiver);
							}
						}
						else {
							return utils::flush(receiver);
						}
					}

					if let Some(details) = event.details::<x::SelectionRequest>() {
						if let Some(c) = utils::flush(receiver) {
							change = c;
						}

						let name      = self.atom(details.target);
						let timestamp = change.0;
						let content   = change.1.clone().into_iter().collect::<BTreeMap<String, Vec<u8>>>();

						match &name[] {
							"TARGETS" => {
								let mut targets: Vec<x::Atom> =
									content.keys().map(|mime| self.intern(&mime[])).collect();

								if content.contains_key("text/plain") {
									targets.push(self.intern("UTF8_STRING"));
								}

								targets.push(self.intern("TARGETS"));
								targets.push(self.intern("TIMESTAMP"));

								self.set(details, &targets);
							},

							"TIMESTAMP" => {
								self.set(details, &vec![timestamp]);
							},

							"UTF8_STRING" | "STRING" => {
								if let Some(value) = content.get("text/plain") {
									self.set(details, value);
								}
								else {
									self.error(details);
								}
							},

							name if regex!(r"^(.*?)/(.*?)$").is_match(name) => {
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

		fn set<T>(&self, details: &x::SelectionRequest, data: &Vec<T>) {
			let window = self.display.window(details.requestor);
			let kind   = match &self.atom(details.target)[] {
				"TARGETS" => self.intern("ATOM"),
				name      => self.intern(name)
			};

			let mut event = self.display.event();
			{
				let notify = event.mut_details::<x::SelectionNotify>();

				notify.requestor = details.requestor;
				notify.selection = details.selection;
				notify.target    = details.target;
				notify.property  = details.property;
				notify.time      = details.time;
			}

			window.set_property(details.property, kind, data);
			window.send(&event);
		}

		fn error(&self, details: &x::SelectionRequest) {
			let window = self.display.window(details.requestor);

			let mut event = self.display.event();
			{
				let notify = event.mut_details::<x::SelectionNotify>();

				notify.requestor = details.requestor;
				notify.selection = details.selection;
				notify.target    = details.target;
				notify.property  = 0;
				notify.time      = details.time;
			}

			window.send(&event);
		}

		pub fn poll(&self) -> Option<clipboard::Change> {
			match self.mode {
				Mode::Selection =>
					self.selection(Mode::Selection).or(self.selection(Mode::Clipboard)),

				Mode::Clipboard =>
					self.selection(Mode::Clipboard)
			}
		}

		fn selection(&self, mode: Mode) -> Option<clipboard::Change> {
			let mut timestamp: u64                       = 0;
			let mut content:   BTreeMap<String, Vec<u8>> = BTreeMap::new();

			let (selection, state) = match mode {
				Mode::Selection => (self.intern("PRIMARY"),   &self.primary),
				Mode::Clipboard => (self.intern("CLIPBOARD"), &self.clipboard),
			};

			if let Some(property) = self.get("UTF8_STRING", selection) {
				content.insert("text/plain".to_string(),
					property.items::<u8>().unwrap().to_vec());
			}
			else if let Some(property) = self.get("STRING", selection) {
				content.insert("text/plain".to_string(),
					property.items::<u8>().unwrap().to_vec());
			}

			if let Some(property) = self.get("TARGETS", selection) {
				for atom in property.items::<x::Atom>().unwrap().iter() {
					let name = self.atom(*atom);

					if regex!(r"^(.*?)/(.*?)$").is_match(&name[]) {
						if let Some(value) = self.get(&name[], selection) {
							if let Some(items) = value.items::<u8>() {
								content.insert(name.clone(), items.to_vec());
							}
						}
					}
					else if "TIMESTAMP" == &name[] {
						timestamp = self.get("TIMESTAMP", selection).unwrap().items::<u32>().unwrap()[0] as u64;
					}
				}
			}

			if content.len() == 0 {
				return None;
			}

			if timestamp != 0 {
				if timestamp == state.timestamp.get() {
					return None;
				}

				state.timestamp.set(timestamp);
			}

			let mut hash: u64 = 0;

			for (ref key, ref value) in content.iter() {
				hash = hash::hash::<_, SipHasher>(&(hash, key, value));
			}

			if hash == state.hash.get() {
				return None;
			}

			state.hash.set(hash);

			Some(Arc::new((timestamp, content.into_iter().collect())))
		}

		fn get(&self, name: &str, selection: x::Atom) -> Option<x::Property> {
			let id     = self.intern("SYMBIOTIC");
			let target = self.intern(name);

			self.window.convert_selection(selection, target, id);

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
		use std::collections::HashMap;

		use std::ops::Drop;

		use std::ptr;
		use std::mem;
		use std::intrinsics::type_id;
		use std::slice;
		use std::cell::RefCell;

		use std::ffi::c_str_to_bytes;
		use std::ffi::CString;

		pub type Id     = c_ulong;
		pub type Atom   = c_ulong;
		pub type Time   = c_ulong;
		pub type Bool   = c_int;
		pub type Status = c_int;

		pub const PropertyChangeMask: c_long = 1 << 22;

		pub const False: Bool = 0;

		pub const CurrentTime: Time = 0;

		pub const PropModeReplace: c_int = 0;

		#[repr(C)]
		pub struct Event {
			pub kind:    c_int,
			pub serial:  c_ulong,
			pub fake:    c_int,
			pub display: *const c_void,

			_data: [u64; 23us]
		}

		#[repr(C)]
		pub struct SelectionNotify {
			pub requestor: Id,
			pub selection: Atom,
			pub target:    Atom,
			pub property:  Atom,
			pub time:      Time,
		}

		#[repr(C)]
		pub struct SelectionClear {
			pub window:    Id,
			pub selection: Atom,
			pub time:      Time,
		}

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

			atoms: RefCell<HashMap<String, Atom>>,
			names: RefCell<HashMap<Atom, String>>,
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
					return None;
				}

				Some(Display {
					pointer: pointer,

					atoms: RefCell::new(HashMap::new()),
					names: RefCell::new(HashMap::new()),
				})
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

			pub fn intern(&self, name: &str) -> Atom {
				if let Some(atom) = self.atoms.borrow_mut().get(&name.to_string()) {
					return *atom;
				}

				let atom = self.intern_atom(name);

				self.atoms.borrow_mut().insert(name.to_string(), atom);

				atom
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

			pub fn atom(&self, atom: Atom) -> String {
				if let Some(name) = self.names.borrow_mut().get(&atom) {
					return name.clone();
				}

				let name = self.atom_name(atom);

				self.names.borrow_mut().insert(atom, name.clone());

				name
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

					data = Vec::from_raw_buf(buffer as *const u8, (items * ((format / 8) as c_ulong)) as usize);

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

			pub fn own_selection(&self, selection: Atom) {
				unsafe {
					XSetSelectionOwner(self.display, selection, self.id, CurrentTime);
				}
			}

			pub fn disown_selection(&self, selection: Atom) {
				unsafe {
					XSetSelectionOwner(self.display, selection, 0, CurrentTime);
				}
			}
		}
	}
}
