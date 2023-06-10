use std::time::Duration;

use clap::Parser;
use winlock::{HotkeyEvent, Key, Modifiers};

#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, clap::Parser)]
struct Options {
	/// Disable the default Windows locking.
	#[arg(short, long)]
	disable_windows: bool,
	/// Restore the default Windows locking at termination.
	///
	/// Tip: if you want to restore it at the start, invoke the program first with just the -r flag.
	#[arg(short, long)]
	restore_windows: bool,
	/// Which key to press (virtual key code number).
	///
	/// Reference: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
	#[arg(short, long)]
	key:             Option<u32>,
	#[arg(short, long)]
	button:          Option<char>,
	/// Control modifier.
	#[arg(short, long)]
	ctrl:            bool,
	/// Shift modifier.
	#[arg(short, long)]
	shift:           bool,
	/// Windows modifier.
	#[arg(short, long)]
	windows:         bool,
	/// Alt modifier.
	#[arg(short, long)]
	alt:             bool,
}

impl Options {
	fn virtual_key(self) -> Option<Key> {
		match (self.key, self.button) {
			(None, None) => None,
			(None, Some(button)) => {
				// XXX: ERROR HANDLE
				Some(Key::from_current_layout_char(button).unwrap())
			}
			(Some(code), None) => Some(Key(code)),
			(Some(_), Some(_)) => unreachable!(),
		}
	}
}

impl From<Options> for Modifiers {
	fn from(value: Options) -> Self {
		let mut result = Self::NoRepeat;
		if value.ctrl {
			result |= Modifiers::Control
		}
		if value.shift {
			result |= Modifiers::Shift
		}
		if value.windows {
			result |= Modifiers::Win
		}
		if value.alt {
			result |= Modifiers::Alt
		}
		result
	}
}

fn main() {
	let options = Options::parse();
	if options.disable_windows {
		winlock::set_lock_enabled(false).unwrap();
	}

	if let Some(key_code) = options.virtual_key() {
		winlock::Hotkey {
			modifiers: Modifiers::from(options),
			key_code,
		}
		.register()
		.unwrap();
		if options.restore_windows {
			ctrlc::set_handler(|| {
				winlock::set_lock_enabled(true).unwrap();
				std::process::exit(0);
			})
			.unwrap();
		}
		loop {
			let event = winlock::await_event().unwrap();
			match event {
				HotkeyEvent::Hotkey => {}
				HotkeyEvent::Other => continue,
				HotkeyEvent::Quit => break,
			}
			winlock::set_lock_enabled(true).unwrap();
			winlock::lock_workstation().unwrap();
			if options.disable_windows {
				// sleep for a bit to avoid race condition (see `set_lock_enabled`'s documentation).
				std::thread::sleep(Duration::from_millis(500));
				winlock::set_lock_enabled(false).unwrap();
			}
		}
	}

	if options.restore_windows {
		winlock::set_lock_enabled(true).unwrap();
	}
}
