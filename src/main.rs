use std::time::Duration;

use clap::Parser;
use tracing_subscriber::EnvFilter;
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
	virtual_code:    Option<u32>,
	#[arg(short, long)]
	/// The key to press.
	///
	/// If it doesn't match to a character, see the -v flag.
	key:             Option<char>,
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

#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, thiserror::Error)]
pub enum OptionsKeyError {
	#[error("failed to map the key to its virtual code")]
	MappingFail,
	#[error("two keyboard shortcuts were given")]
	Conflict,
}

impl Options {
	fn virtual_key(self) -> Result<Option<Key>, OptionsKeyError> {
		match (self.virtual_code, self.key) {
			(None, None) => Ok(None),
			(None, Some(button)) => Key::from_current_layout_char(button)
				.map_or(Err(OptionsKeyError::MappingFail), |k| Ok(Some(k))),
			(Some(code), None) => Ok(Some(Key(code))),
			(Some(_), Some(_)) => Err(OptionsKeyError::Conflict),
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

fn disable_lock() {
	if let Err(e) = winlock::set_lock_enabled(false) {
		tracing::error!("failed to disable locking: {e}");
	}
}

fn enable_lock() {
	if let Err(e) = winlock::set_lock_enabled(true) {
		tracing::error!("failed to restore locking: {e}");
	}
}

impl Options {
	fn cleanup(self) {
		if self.restore_windows {
			enable_lock();
		}
	}
}

fn main() {
	let options = Options::parse();

	{
		let subscriber = tracing_subscriber::fmt()
			.with_env_filter(
				EnvFilter::builder()
					.with_env_var("WINLOCK_LOG")
					.from_env_lossy(),
			)
			.finish();
		let _ = tracing::subscriber::set_global_default(subscriber)
			.map_err(|e| eprintln!("failed to set up logging: {e}"));
	}

	if options.disable_windows {
		disable_lock();
	}

	match options.virtual_key() {
		Ok(Some(key_code)) => {
			let register_result = winlock::Hotkey {
				modifiers: Modifiers::from(options),
				key_code,
			}
			.register();
			if let Err(e) = register_result {
				tracing::error!("failed to register the hotkey in the system: {e}, terminating.");
				options.cleanup();
				std::process::exit(1);
			}
			if options.restore_windows {
				let _ = ctrlc::set_handler(move || {
					options.cleanup();
					std::process::exit(0);
				})
				.map_err(|e| tracing::warn!("failed to hook restoration on termination: {e}"));
			}
			loop {
				let event = match winlock::await_event() {
					Ok(event) => event,
					Err(error) => {
						tracing::error!("failed to listen to a message from Windows: {error}");
						continue;
					}
				};
				match event {
					HotkeyEvent::Hotkey => {}
					HotkeyEvent::Other => continue,
					HotkeyEvent::Quit => break,
				}
				enable_lock();
				if let Err(e) = winlock::lock_workstation() {
					tracing::error!("failed to lock the workstation: {e}");
				}
				if options.disable_windows {
					// sleep for a bit to avoid race condition (see `set_lock_enabled`'s documentation).
					std::thread::sleep(Duration::from_millis(500));
					disable_lock();
				}
			}
		}
		Ok(None) => {}
		Err(e) => {
			tracing::error!("{e}");
			std::process::exit(1);
		}
	}

	if options.restore_windows {
		enable_lock();
	}
}
