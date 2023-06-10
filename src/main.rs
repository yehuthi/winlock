fn main() {
	winlock::Hotkey {
		modifiers: winlock::Modifiers::Win | winlock::Modifiers::Control,
		key_code:  0x4A,
	}
	.register()
	.unwrap();
	winlock::handle_event().unwrap();
	winlock::lock_workstation().unwrap();
}
