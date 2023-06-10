use std::io;

use bitflags::bitflags;
use windows::Win32::{
	Foundation::HWND,
	UI::{
		Input::KeyboardAndMouse::{RegisterHotKey, HOT_KEY_MODIFIERS},
		WindowsAndMessaging::{GetMessageW, WM_HOTKEY},
	},
};

#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Config {
	hotkey: Option<Hotkey>,
}

bitflags! {
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Modifiers: u32 {
	const Alt = 0x0001;
	const Control = 0x0002;
	const NoRepeat = 0x4000;
	const Shift = 0x0004;
	const Win = 0x0008;
}
}

impl From<Modifiers> for HOT_KEY_MODIFIERS {
	fn from(value: Modifiers) -> Self { Self(value.bits()) }
}

#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hotkey {
	pub modifiers: Modifiers,
	// https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
	pub key_code:  u32,
}

impl Hotkey {
	pub fn register(self) -> io::Result<()> {
		let success = unsafe {
			RegisterHotKey(
				HWND::default(),
				0x31710C4,
				self.modifiers.into(),
				self.key_code,
			)
			.as_bool()
		};
		if success {
			Ok(())
		} else {
			Err(std::io::Error::last_os_error())
		}
	}
}

pub fn handle_event() -> io::Result<bool> {
	let mut message = Default::default();
	let message_result = unsafe { GetMessageW(&mut message, HWND::default(), 0, 0) };
	let message = match message_result.0 {
		0 => return Ok(true),
		-1 => return Err(io::Error::last_os_error()),
		_ => message,
	};
	Ok(message.message == WM_HOTKEY)
}

pub fn lock_workstation() -> io::Result<()> {
	let result = unsafe { windows::Win32::System::Shutdown::LockWorkStation() }.as_bool();
	if result {
		Ok(())
	} else {
		Err(io::Error::last_os_error())
	}
}
