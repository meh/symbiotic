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

use std::thread::Thread;
use std::sync::mpsc::{Sender, channel};
use std::sync::Arc;

use std::io::timer::sleep;
use std::time::duration::Duration;

use clipboard;
use clipboard::Direction::Outgoing;

use utils;

pub fn start(main: Sender<clipboard::Message>, _: Option<toml::Value>) -> Sender<clipboard::Change> {
	let (sender, receiver) = channel::<clipboard::Change>();

	Thread::spawn(move || -> () {
		let mut sequence = win::sequence();

		loop {
			if let Some(change) = utils::flush(&receiver) {
				{
					let clipboard = win::Clipboard::open();

					clipboard.empty();

					for &(ref name, ref value) in change.1.iter() {
						clipboard.set(name, value);
					}
				}

				sequence = win::sequence();
			}
			else {
				let current = win::sequence();

				if current == sequence {
					// this could be reduced a bit, but unsure about it
					sleep(Duration::seconds(1));

					continue;
				}

				sequence = current;

				{
					let clipboard = win::Clipboard::open();

					main.send(Outgoing(Arc::new((sequence as clipboard::Timestamp, clipboard.get())))).unwrap();
				}
			}
		}
	});

	sender
}

#[allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
mod win {
	extern crate libc;
	extern crate unicode;
	extern crate image;

	use self::image::{GenericImage, ImageFormat, Rgba};
	use self::image::DynamicImage::{ImageLuma8, ImageLumaA8, ImageRgb8, ImageRgba8};

	use self::libc::{c_int, c_uint, c_long, uintptr_t, c_char};
	use self::libc::types::os::arch::extra::{BYTE, WORD, DWORD, BOOL, HANDLE, LONG_PTR, SIZE_T};

	use std::ffi::c_str_to_bytes;
	use std::ffi::CString;

	use std::io::ByRefWriter;

	use std::ptr;
	use std::mem;
	use std::slice;
	use std::str;

	use std::ops::Deref;

	type UINT = c_uint;
	type LONG = c_long;

	type UINT_PTR = uintptr_t;

	type HWND      = HANDLE;
	type HMENU     = HANDLE;
	type HINSTANCE = HANDLE;
	type HGLOBAL   = HANDLE;

	type LPVOID  = HANDLE;
	type LPCTSTR = *const c_char;
	type LPTSTR  = *mut c_char;
	type LPMSG   = HANDLE;

	type WPARAM = LONG_PTR;
	type LPARAM = UINT_PTR;

	const GHND: UINT = 0x0042;

	const CF_BITMAP: UINT          = 2;
	const CF_DIB: UINT             = 8;
	const CF_DIBV5: UINT           = 17;
	const CF_DIF: UINT             = 5;
	const CF_DSPBITMAP: UINT       = 0x0082;
	const CF_DSPENHMETAFILE: UINT  = 0x008E;
	const CF_DSPMETAFILEPICT: UINT = 0x0083;
	const CF_DSPTEXT: UINT         = 0x0081;
	const CF_ENHMETAFILE: UINT     = 14;
	const CF_GDIOBJFIRST: UINT     = 0x0300;
	const CF_GDIOBJLAST: UINT      = 0x03FF;
	const CF_HDROP: UINT           = 15;
	const CF_LOCALE: UINT          = 16;
	const CF_METAFILEPICT: UINT    = 3;
	const CF_OEMTEXT: UINT         = 7;
	const CF_OWNERDISPLAY: UINT    = 0x0080;
	const CF_PALETTE: UINT         = 9;
	const CF_PENDATA: UINT         = 10;
	const CF_PRIVATELAST: UINT     = 0x02FF;
	const CF_RIFF: UINT            = 11;
	const CF_SYLK: UINT            = 4;
	const CF_TEXT: UINT            = 1;
	const CF_TIFF: UINT            = 6;
	const CF_UNICODETEXT: UINT     = 13;
	const CF_WAVE: UINT            = 12;

	struct Global(HGLOBAL);

	struct Lock(LPVOID);

	impl Global {
		pub fn new(size: usize) -> Self {
			unsafe {
				Global(GlobalAlloc(GHND, size as SIZE_T))
			}
		}

		pub unsafe fn get(&self) -> HGLOBAL {
			self.0
		}

		pub fn is_null(&self) -> bool {
			self.0.is_null()
		}

		pub fn size(&self) -> usize {
			unsafe {
				GlobalSize(self.0) as usize
			}
		}

		pub fn lock(&self) -> Lock {
			Lock::new(self.0)
		}
	}

	impl Lock {
		pub fn new(ptr: HGLOBAL) -> Self {
			unsafe {
				Lock(GlobalLock(ptr))
			}
		}
	}

	impl Deref for Lock {
		type Target = LPVOID;

		fn deref<'a>(&'a self) -> &'a LPVOID {
			&self.0
		}
	}

	impl Drop for Lock {
		fn drop(&mut self) {
			unsafe {
				GlobalUnlock(self.0);
			}
		}
	}

	#[repr(C)]
	struct RGBQUAD {
		rgbBlue:     BYTE,
		rgbGreen:    BYTE,
		rgbRed:      BYTE,
		rgbReserved: BYTE,
	}

	#[repr(C)]
	struct BITMAPINFOHEADER {
		biSize:          DWORD,
		biWidth:         LONG,
		biHeight:        LONG,
		biPlanes:        WORD,
		biBitCount:      WORD,
		biCompression:   DWORD,
		biSizeImage:     DWORD,
		biXPelsPerMeter: LONG,
		biYPelPerMeter:  LONG,
		biClrUsed:       DWORD,
		biClrImportant:  DWORD,
	}

	#[repr(C)]
	struct BITMAPINFO {
		bmiHeader: BITMAPINFOHEADER,
		bmiColors: [RGBQUAD; 1],
	}

	#[link(name = "kernel32")]
	extern "system" {
		fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
		fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
		fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
		fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;
	}

	#[link(name = "user32")]
	extern "system" {
		fn GetClipboardSequenceNumber() -> DWORD;

		fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
		fn CloseClipboard() -> BOOL;
		fn EmptyClipboard() -> BOOL;

		fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE;
		fn GetClipboardData(uFormat: UINT) -> HANDLE;

		fn EnumClipboardFormats(format: UINT) -> UINT;
		fn RegisterClipboardFormatA(lpszFormat: LPCTSTR) -> UINT;
		fn GetClipboardFormatNameA(format: UINT, lpszFormatName: LPTSTR, cchMaxCount: c_int) -> c_int;
	}

	pub fn sequence() -> DWORD {
		unsafe {
			GetClipboardSequenceNumber()
		}
	}

	fn strlen(ptr: HANDLE) -> usize {
		unsafe {
			let mut length = 0;
			let mut ptr    = ptr as *const u16;

			while *ptr != 0 {
				length += 1;
				ptr     = ptr.offset(1);
			}

			length
		}
	}

	pub struct Clipboard(());

	impl Clipboard {
		pub fn open() -> Clipboard {
			unsafe {
				OpenClipboard(0 as HWND);
			}

			Clipboard(())
		}

		pub fn empty(&self) {
			unsafe {
				EmptyClipboard();
			}
		}

		fn register(&self, name: &str) -> UINT {
			unsafe {
				RegisterClipboardFormatA(CString::from_slice(name.as_bytes()).as_ptr())
			}
		}

		fn name(&self, format: UINT) -> String {
			match format {
				CF_BITMAP =>
					"CF_BITMAP".to_string(),

				CF_DIB =>
					"CF_DIB".to_string(),

				CF_DIBV5 =>
					"CF_DIBV5".to_string(),

				CF_DIF =>
					"CF_DIF".to_string(),

				CF_DSPBITMAP =>
					"CF_DSPBITMAP".to_string(),

				CF_DSPENHMETAFILE =>
					"CF_DSPENHMETAFILE".to_string(),

				CF_DSPMETAFILEPICT =>
					"CF_DSPMETAFILEPICT".to_string(),

				CF_DSPTEXT =>
					"CF_DSPTEXT".to_string(),

				CF_ENHMETAFILE =>
					"CF_ENHMETAFILE".to_string(),

				CF_GDIOBJFIRST =>
					"CF_GDIOBJFIRST".to_string(),

				CF_GDIOBJLAST =>
					"CF_GDIOBJLAST".to_string(),

				CF_HDROP =>
					"CF_HDROP".to_string(),

				CF_LOCALE =>
					"CF_LOCALE".to_string(),

				CF_METAFILEPICT =>
					"CF_METAFILEPICT".to_string(),

				CF_OEMTEXT =>
					"CF_OEMTEXT".to_string(),

				CF_OWNERDISPLAY =>
					"CF_OWNERDISPLAY".to_string(),

				CF_PALETTE =>
					"CF_PALETTE".to_string(),

				CF_PENDATA =>
					"CF_PENDATA".to_string(),

				CF_PRIVATELAST =>
					"CF_PRIVATELAST".to_string(),

				CF_RIFF =>
					"CF_RIFF".to_string(),

				CF_SYLK =>
					"CF_SYLK".to_string(),

				CF_TEXT =>
					"CF_TEXT".to_string(),

				CF_TIFF =>
					"CF_TIFF".to_string(),

				CF_UNICODETEXT =>
					"CF_UNICODETEXT".to_string(),

				CF_WAVE =>
					"CF_WAVE".to_string(),

				format => unsafe {
					let mut buffer: [i8; 256] = mem::zeroed();

					GetClipboardFormatNameA(format, buffer.as_mut_ptr() as LPTSTR, 255);

					String::from_utf8_lossy(c_str_to_bytes(&buffer.as_ptr())).into_owned()
				}
			}
		}

		pub fn set(&self, name: &String, data: &Vec<u8>) {
			let format = match &name[] {
				"text/plain" =>
					CF_UNICODETEXT,

				n if n.starts_with("image/") =>
					CF_DIB,

				n =>
					self.register(n)
			};

			let handle = match &name[] {
				n if n.starts_with("text/") => {
					if data.iter().any(|x| *x == 0) {
						if unicode::str::is_utf16(unsafe { slice::from_raw_buf(&(data.as_ptr() as *const u16), data.len() / 2) }) {
							debug!("input: {}: utf16", name);

							unsafe {
								let handle = Global::new(data.len() + 2);
								let lock   = handle.lock();

								ptr::copy_memory(*lock as *mut u8, (&data[]).as_ptr(), data.len());

								handle
							}
						}
						else {
							debug!("input: {}: unknown multi-byte", name);

							unsafe {
								let handle = Global::new(data.len() + 4);
								let lock   = handle.lock();

								ptr::copy_memory(*lock as *mut u8, (&data[]).as_ptr(), data.len());

								handle
							}
						}
					}
					else {
						if let Ok(string) = str::from_utf8(&data[]) {
							debug!("input: {}: utf8", name);

							let data = string.utf16_units().collect::<Vec<u16>>();

							unsafe {
								let handle = Global::new(data.len() * 2 + 2);
								let lock   = handle.lock();

								ptr::copy_memory(*lock as *mut u16, (&data[]).as_ptr(), data.len());

								handle
							}
						}
						else {
							debug!("input: {}: unknown", name);

							unsafe {
								let handle = Global::new(data.len() + 1);
								let lock   = handle.lock();

								ptr::copy_memory(*lock as *mut u8, (&data[]).as_ptr(), data.len());

								handle
							}
						}
					}
				},

				n if n.starts_with("image/") => {
					let format = match &n[] {
						"image/png"   => ImageFormat::PNG,
						"image/jpeg"  => ImageFormat::JPEG,
						"image/gif"   => ImageFormat::GIF,
						"image/tiff"  => ImageFormat::TIFF,
						"image/x-tga" => ImageFormat::TGA,
						"image/webp"  => ImageFormat::WEBP,

						_ => return
					};

					let image = image::load_from_memory_with_format(&data[], format);

					if image.is_err() {
						return;
					}

					let image           = image.unwrap();
					let (width, height) = match image.dimensions() {
						(width, height) => (width as usize, height as usize)
					};

					unsafe {
						let handle = Global::new(mem::size_of::<BITMAPINFO>() + (width * height * 4));

						{
							let lock = handle.lock();
							let info = &mut (*(*lock as *mut BITMAPINFO)).bmiHeader;

							info.biSize = mem::size_of::<BITMAPINFOHEADER>() as DWORD;

							info.biWidth  = width as LONG;
							info.biHeight = height as LONG;

							info.biPlanes   = 1;
							info.biBitCount = match image {
								ImageLuma8(..)  | ImageRgb8(..)  => 24,
								ImageLumaA8(..) | ImageRgba8(..) => 32,
							};

							info.biSizeImage     = 0;
							info.biXPelsPerMeter = 0;
							info.biYPelPerMeter  = 0;
							info.biClrUsed       = 0;
							info.biClrImportant  = 0;
						}

						{
							let lock = handle.lock();
							let ptr  = (*(*lock as *mut BITMAPINFO)).bmiColors.as_mut_ptr();
							let data = slice::from_raw_mut_buf::<RGBQUAD>(&ptr, width * height);

							for (i, &Rgba([r, g, b, a])) in image.to_rgba().pixels().enumerate() {
								let x = i % width;
								let y = i / width;
								let i = ((height - y - 1) * width) + x;

								data[i].rgbBlue     = b;
								data[i].rgbGreen    = g;
								data[i].rgbRed      = r;
								data[i].rgbReserved = a;
							}
						}

						handle
					}
				},

				_ => {
					unsafe {
						let handle = Global::new(data.len());
						let lock   = handle.lock();

						ptr::copy_memory(*lock as *mut u8, (&data[]).as_ptr() as *mut u8, data.len());

						handle
					}
				}
			};

			unsafe {
				SetClipboardData(format, handle.get());
			}
		}

		pub fn get(&self) -> Vec<(String, Vec<u8>)> {
			let mut result = vec!();

			unsafe {
				let mut format = 0;

				while { format = EnumClipboardFormats(format); format != 0 } {
					let handle = Global(GetClipboardData(format));

					if handle.is_null() {
						continue;
					}

					let lock = handle.lock();

					match &self.name(format)[] {
						"CF_UNICODETEXT" => {
							result.push(("text/plain".to_string(),
								String::from_utf16(
									slice::from_raw_buf(
										&(*lock as *const u16), strlen(*lock))).unwrap().as_bytes().to_vec()));
						},

						"CF_DIB" => {
							let info   = &mut (*(*lock as *mut BITMAPINFO)).bmiHeader;
							let width  = info.biWidth as usize;
							let height = info.biHeight as usize;

							if info.biBitCount == 32 || info.biBitCount == 24 {
								let     ptr  = (*(*lock as *mut BITMAPINFO)).bmiColors.as_mut_ptr();
								let     data = slice::from_raw_mut_buf::<RGBQUAD>(&ptr, (width * height) as usize);
								let mut buf  = image::ImageBuffer::new(width as u32, height as u32);

								for (i, px) in data.iter().enumerate() {
									let x = i % width;
									let y = i / width;

									match info.biBitCount {
										32 => buf.put_pixel(x as u32, (height - y - 1) as u32,
											Rgba([px.rgbRed, px.rgbGreen, px.rgbBlue, px.rgbReserved])),

										24 => buf.put_pixel(x as u32, (height - y - 1) as u32,
											Rgba([px.rgbRed, px.rgbGreen, px.rgbBlue, 255])),

										_ => {}
									}
								}

								let mut data: Vec<u8> = vec!();

								if let Ok(..) = ImageRgba8(buf).save(data.by_ref(), image::PNG) {
									result.push(("image/png".to_string(), data));
								}
							}
						},

						name if name.starts_with("text/") => {
							result.push((name.to_string(),
								slice::from_raw_buf(&(*lock as *const u8), strlen(*lock) * 2).to_vec()));
						},

						name if regex!(r"^(.*?)/(.*?)$").is_match(name) => {
							result.push((name.to_string(),
								slice::from_raw_buf(&(*lock as *const u8), handle.size()).to_vec()));
						},

						_ => {}
					}
				}
			}

			result
		}
	}

	impl Drop for Clipboard {
		fn drop(&mut self) {
			unsafe {
				CloseClipboard();
			}
		}
	}
}
