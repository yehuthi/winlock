fn main() {
	winlock::Hotkey {
		modifiers: winlock::Modifiers::Win | winlock::Modifiers::Control,
		key_code:  0x4A,
	}
	.register()
	.unwrap();
	while let false = winlock::handle_event().unwrap() {}
	winlock::lock_workstation().unwrap();
}
