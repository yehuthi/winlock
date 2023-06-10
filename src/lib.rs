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
/// Keyboard [`Hotkey`] modifiers.
///
/// Corresponds with [`RegisterHotKey`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)'s [`fsModifiers`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey#:~:text=the%20action%20taken.-,%5Bin%5D%20fsModifiers,-Type%3A%20UINT) parameter.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Modifiers: u32 {
	/// Either <kbd>ALT</kbd> key must be held down.
	const Alt = 0x0001;
	/// Either <kbd>CTRL</kbd> key must be held down.
	const Control = 0x0002;
	/// Changes the hotkey behavior so that the keyboard auto-repeat does not yield multiple hotkey notifications.
	///
	/// Not supported on Windows Vista.
	const NoRepeat = 0x4000;
	/// Either <kbd>SHIFT</kbd> key must be held down.
	const Shift = 0x0004;
	/// Either <kbd>WINDOWS</kbd> key was held down.
	const Win = 0x0008;
}
}

impl From<Modifiers> for HOT_KEY_MODIFIERS {
	fn from(value: Modifiers) -> Self { Self(value.bits()) }
}

/// A global keyboard hotkey / shortcut that can be [`register`](Hotkey::register)ed.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hotkey {
	/// The hotkey [`Modifiers`].
	pub modifiers: Modifiers,
	/// The hotkey's [virtual key code](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
	pub key_code:  u32,
}

impl Hotkey {
	/// Registers the [`Hotkey`].
	///
	/// This procedure just has the system notify the application when the hotkey is pressed with a message.
	/// Actually handling it is done in the message loop (see [`handle_event`]).
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

/// Blocks until the next Windows message, returns true if the message was a hotkey press.
pub fn handle_event() -> io::Result<bool> {
	let mut message = Default::default();
	let message_result =
		unsafe { GetMessageW(&mut message, HWND::default(), WM_HOTKEY, WM_HOTKEY) };
	let message = match message_result.0 {
		0 => return Ok(true),
		-1 => return Err(io::Error::last_os_error()),
		_ => message,
	};
	Ok(message.message == WM_HOTKEY)
}

/// Locks the workstation / user session.
///
/// Corresponds to [LockWorkStation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-lockworkstation).
pub fn lock_workstation() -> io::Result<()> {
	let result = unsafe { windows::Win32::System::Shutdown::LockWorkStation() }.as_bool();
	if result {
		Ok(())
	} else {
		Err(io::Error::last_os_error())
	}
}
