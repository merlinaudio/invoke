use objc2_core_graphics::{CGEvent, CGEventTapLocation};

pub mod keyboard;
pub mod mouse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Event could not be created")]
	EventCreationFailed,

	#[error("Invalid PID")]
	InvalidPid(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventLocation {
	/// Event gets created at the HID level, before other processes (e.g. Mac Mouse Fix, Karabiner, ...) can see/modify it.
	///
	/// Use `EventLocation::Process` by default or if unsure, as `Hid` cannot simulate all keys, e.g. Unicode strings, and can be modified by other processes on accident,
	/// such as apps that change mouse scrolling behavior (Mac Mouse Fix), keyboard remapping apps (Karabiner), etc.
	///
	/// These apps will usually not be able to modify the event if sent with `Process`.
	Hid,
	/// Event gets created at the process level.
	///
	/// Use this by default or if unsure.
	///
	/// `Process` may have some shortcomings, such as not being able to simulate a "press and hold" style event.
	/// But try using `Process` first and explore other options only if you run into issues.
	Process { pid: u32 },
}

impl EventLocation {
	pub fn post(&self, event: &CGEvent) -> Result<(), Error> {
		match self {
			EventLocation::Hid => CGEvent::post(CGEventTapLocation::HIDEventTap, Some(event)),
			EventLocation::Process { pid } => {
				let pid = *pid;
				let pid = i32::try_from(pid).map_err(|_| Error::InvalidPid(pid))?;
				CGEvent::post_to_pid(pid, Some(event))
			}
		}

		Ok(())
	}
}
