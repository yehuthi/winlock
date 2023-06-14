#![doc(
	html_logo_url = "https://github.com/yehuthi/winlock/blob/0a5dd2c85d33b8d1e8e906ee48491a6cb186c174/winlock.svg?raw=true"
)]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[cfg(not(target_family = "windows"))]
compile_error!("This library targets Windows only.");

use std::{io, mem};

use bitflags::bitflags;
use windows::{
	w,
	Win32::{
		Foundation::HWND,
		System::Registry::{RegSetKeyValueW, HKEY_CURRENT_USER, REG_DWORD},
		UI::{
			Input::KeyboardAndMouse::{RegisterHotKey, VkKeyScanW, HOT_KEY_MODIFIERS},
			WindowsAndMessaging::{GetMessageW, WM_HOTKEY, WM_QUIT},
		},
	},
};

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

#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
/// A keyboard key / button.
pub struct Key(
	/// The key's [virtual key code](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
	pub u32,
);

impl Key {
	/// Converts a [`char`] into a [`Key`].
	///
	/// It reads the user's current keyboard layout to determine the key that emits the given character.
	///
	/// Corresponds to [VkKeyScanW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-vkkeyscanw).
	pub fn from_current_layout_char(c: char) -> Option<Self> {
		let scan = unsafe { VkKeyScanW(c as u16) };
		let vkc = scan & 0x00FF;
		(vkc != -1).then_some(Key(vkc as u32))
	}
}

/// A global keyboard hotkey / shortcut that can be [`register`](Hotkey::register)ed.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hotkey {
	/// The hotkey [`Modifiers`].
	pub modifiers: Modifiers,
	/// The hotkey's [virtual key code](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
	pub key_code:  Key,
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
				self.key_code.0,
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

/// An event from the Windows message loop in the context of hotkeys.
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HotkeyEvent {
	/// The hotkey was pressed.
	Hotkey,
	/// An irrelevant event occurred.
	Other,
	/// Got a quit signal.
	Quit,
}

/// Blocks until the next Windows message.
pub fn await_event() -> io::Result<HotkeyEvent> {
	let mut message = Default::default();
	let message_result =
		unsafe { GetMessageW(&mut message, HWND::default(), WM_HOTKEY, WM_HOTKEY) };
	let message = match message_result.0 {
		0 => return Ok(HotkeyEvent::Quit),
		-1 => return Err(io::Error::last_os_error()),
		_ => message,
	};
	match message.message {
		WM_HOTKEY => Ok(HotkeyEvent::Hotkey),
		WM_QUIT => Ok(HotkeyEvent::Quit),
		_ => Ok(HotkeyEvent::Quit),
	}
}

/// Locks the workstation / user session.
///
/// This procedure can return a successful result but not have the workstation locked. This can happen for details specified in the
/// Windows API documentation linked below, or because [workstation locking is disabled](set_lock_enabled).
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

/// Sets whether to enable or disable workstation locking.
///
/// If disabled, it's impossible to lock the workstation, whether by shortcut (<kbd>Wind</kbd> + <kbd>L</kbd>) or programmatically ([`lock_workstation`]).
/// Note that attempting to lock and immediately disabling locking afterwards is a race condition, and locking will likely be disabled by the time the issued locking begins,
/// therefore preventing the computer from locking altogether.
///
/// This procedure achieves its behavior by modifying the Windows registry so expect this to only work with elevated privileges.
pub fn set_lock_enabled(enabled: bool) -> io::Result<()> {
	let data: u32 = if enabled { 0 } else { 1 };
	let result = unsafe {
		RegSetKeyValueW(
			HKEY_CURRENT_USER,
			w!(r"Software\Microsoft\Windows\CurrentVersion\Policies\System"),
			w!(r"DisableLockWorkstation"),
			REG_DWORD.0,
			Some(&data as *const _ as *const _),
			mem::size_of_val(&data) as _,
		)
	};
	if result.is_ok() {
		Ok(())
	} else {
		Err(io::Error::from_raw_os_error(result.0 as _))
	}
}
