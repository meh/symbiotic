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

#[allow(non_camel_case_types)]
mod ffi {
  use libc::{c_uint, uintptr_t};
  use libc::types::os::arch::extra::{BOOL, HANDLE, LONG_PTR};

  type UINT = c_uint;
  type UINT_PTR = uintptr_t;
  type HWND = HANDLE;
  type WPARAM = UINT_PTR;
  type LPARAM = LONG_PTR;

  pub static HWND_BROADCAST: HWND = 0xffff as HWND;
  pub static WM_SYSCOMMAND: UINT = 0x0112;
  pub static SC_MONITORPOWER: WPARAM = 0xf170;

  #[link(name = "user32")]
  extern "stdcall" {
    pub fn SendNotifyMessageW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
  }
}

pub struct Clipboard {
	x: int,
}

impl Clipboard {
	pub fn new() {
		Clipboard { x: 2 }
	}
}

/*impl clipboard::Clipboard for Clipboard {*/
/*}*/
