use std::fmt::Debug;

use enumset::{EnumSet, EnumSetType};
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::{CGEvent, CGEventFlags};

use crate::{Error, EventLocation};

#[derive(PartialOrd, Ord, Hash, EnumSetType)]
#[cfg_attr(debug_assertions, derive(Debug))] // Only log actual Key in debug mode
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "napi", napi_derive::napi)]
pub enum Key {
	// Function Keys
	F1 = 122,
	F2 = 120,
	F3 = 99,
	F4 = 118,
	F5 = 96,
	F6 = 97,
	F7 = 98,
	F8 = 100,
	F9 = 101,
	F10 = 109,
	F11 = 103,
	F12 = 111,
	F13 = 105,
	F14 = 107,
	F15 = 113,
	F16 = 106,
	F17 = 64,
	F18 = 79,
	F19 = 80,
	F20 = 90,

	// Special Keys
	#[serde(alias = "esc")]
	Escape = 53,
	Delete = 51,
	Tab = 48,
	#[serde(alias = "caps", alias = "alphaShift")]
	CapsLock = 57,
	Return = 36,
	Space = 49,

	// Arrow Keys
	#[serde(alias = "left")]
	LeftArrow = 123,
	#[serde(alias = "right")]
	RightArrow = 124,
	#[serde(alias = "up")]
	UpArrow = 126,
	#[serde(alias = "down")]
	DownArrow = 125,

	// Navigation Keys
	PageUp = 116,
	PageDown = 121,
	Home = 115,
	End = 119,

	// Modifier Keys
	#[serde(alias = "shift")]
	LeftShift = 56,
	RightShift = 60,
	#[serde(alias = "control", alias = "ctrl")]
	LeftControl = 59,
	#[serde(alias = "rightCtrl")]
	RightControl = 62,
	#[serde(alias = "option", alias = "alternate", alias = "opt", alias = "alt")]
	LeftOption = 58,
	RightOption = 61,
	#[serde(alias = "command", alias = "cmd")]
	LeftCommand = 55,
	#[serde(alias = "rightCmd")]
	RightCommand = 54,

	#[serde(alias = "fn")]
	Globe = 63,

	// Letters
	A = 0,
	B = 11,
	C = 8,
	D = 2,
	E = 14,
	F = 3,
	G = 5,
	H = 4,
	I = 34,
	J = 38,
	K = 40,
	L = 37,
	M = 46,
	N = 45,
	O = 31,
	P = 35,
	Q = 12,
	R = 15,
	S = 1,
	T = 17,
	U = 32,
	V = 9,
	W = 13,
	X = 7,
	Y = 16,
	Z = 6,

	// Numbers (top row)
	#[serde(rename = "1", alias = "num1")]
	Num1 = 18,
	#[serde(rename = "2", alias = "num2")]
	Num2 = 19,
	#[serde(rename = "3", alias = "num3")]
	Num3 = 20,
	#[serde(rename = "4", alias = "num4")]
	Num4 = 21,
	#[serde(rename = "5", alias = "num5")]
	Num5 = 23,
	#[serde(rename = "6", alias = "num6")]
	Num6 = 22,
	#[serde(rename = "7", alias = "num7")]
	Num7 = 26,
	#[serde(rename = "8", alias = "num8")]
	Num8 = 28,
	#[serde(rename = "9", alias = "num9")]
	Num9 = 25,
	#[serde(rename = "0", alias = "num0")]
	Num0 = 29,

	// Symbols
	#[serde(alias = "~", alias = "`", alias = "backtick")]
	Grave = 50, // ~ `
	#[serde(alias = "{", alias = "[")]
	LeftBracket = 33, // { [
	#[serde(alias = "}", alias = "]")]
	RightBracket = 30, // } ]
	#[serde(alias = "|", alias = "\\")]
	Backslash = 42, // | \
	#[serde(alias = ":", alias = ";")]
	Semicolon = 41, // : ;
	#[serde(alias = "'", alias = "\"")]
	Quote = 39, // " '
	#[serde(alias = "-", alias = "_")]
	Minus = 27, // _ -
	#[serde(alias = "+", alias = "=")]
	Equal = 24, // + =
	#[serde(alias = "<", alias = ",")]
	Comma = 43, // < ,
	#[serde(alias = ">", alias = ".")]
	Period = 47, // > .
	#[serde(alias = "?", alias = "/")]
	Slash = 44, // ? /
	#[serde(alias = "§", alias = "intlBackslash", alias = "internationalBackslash", alias = "intl\\")]
	Section = 10, // § (EU/ISO keyboards only)

	// Numpad
	Numpad0 = 82,
	Numpad1 = 83,
	Numpad2 = 84,
	Numpad3 = 85,
	Numpad4 = 86,
	Numpad5 = 87,
	Numpad6 = 88,
	Numpad7 = 89,
	Numpad8 = 91,
	Numpad9 = 92,
	NumpadMultiply = 67,
	NumpadAdd = 69,
	NumpadSubtract = 78,
	NumpadDivide = 75,
	NumpadEnter = 76,   // Numeric keypad Enter
	NumpadDecimal = 65, // Numpad Decimal (on PC)
	NumpadClear = 71,   // Numpad Clear (on PC)
	NumpadComma = 95,   // Keypad Comma/Separator (JIS layout)

	VolumeUp = 72,       // Volume Up
	VolumeDown = 73,     // Volume Down
	Mute = 74,           // Mute
	Yen = 93,            // Yen (JIS layout)
	Underscore = 94,     // Underscore (JIS layout)
	Eisu = 102,          // Eisu (JIS layout)
	Kana = 104,          // Kana (JIS layout)
	Menu = 110,          // Menu (on PC)
	Help = 114,          // Help
	ForwardDelete = 117, // Forward Delete (Del below Help, usually not used because Fn+Delete is conventionally used on macOS)
	Power = 127,         // Power (on PC)
	MissionControl = 160,

	// Unused
	EnterPowerbook = 52, // Enter key (Powerbook keyboards only), unused

	                     // If adding more keys (especially below `127`, please add them in TryFrom below.)
}

// In production, non-debug_assertions, we don't log the actual key because this may contain sensitive information.
// For example, if we log a series of HID events, accidentally logging the Key too -- that's a keylogger.
#[cfg(not(debug_assertions))]
impl Debug for Key {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Key")
	}
}

impl TryFrom<u16> for Key {
	type Error = ();

	fn try_from(unichar: u16) -> Result<Key, Self::Error> {
		use Key::*;
		Ok(match unichar {
			// Function Keys
			122 => F1,
			120 => F2,
			99 => F3,
			118 => F4,
			96 => F5,
			97 => F6,
			98 => F7,
			100 => F8,
			101 => F9,
			109 => F10,
			103 => F11,
			111 => F12,
			105 => F13,
			107 => F14,
			113 => F15,
			106 => F16,
			64 => F17,
			79 => F18,
			80 => F19,
			90 => F20,

			// Special Keys
			53 => Escape,
			51 => Delete,
			48 => Tab,
			57 => CapsLock,
			36 => Return,
			49 => Space,

			// Arrow Keys
			123 => LeftArrow,
			124 => RightArrow,
			126 => UpArrow,
			125 => DownArrow,

			// Navigation Keys
			116 => PageUp,
			121 => PageDown,
			115 => Home,
			119 => End,

			// Modifier Keys
			56 => LeftShift,
			60 => RightShift,
			59 => LeftControl,
			62 => RightControl,
			58 => LeftOption,
			61 => RightOption,
			55 => LeftCommand,
			54 => RightCommand,

			63 => Globe,

			// Letters
			0 => A,
			11 => B,
			8 => C,
			2 => D,
			14 => E,
			3 => F,
			5 => G,
			4 => H,
			34 => I,
			38 => J,
			40 => K,
			37 => L,
			46 => M,
			45 => N,
			31 => O,
			35 => P,
			12 => Q,
			15 => R,
			1 => S,
			17 => T,
			32 => U,
			9 => V,
			13 => W,
			7 => X,
			16 => Y,
			6 => Z,

			// Numbers (top row)
			18 => Num1,
			19 => Num2,
			20 => Num3,
			21 => Num4,
			23 => Num5,
			22 => Num6,
			26 => Num7,
			28 => Num8,
			25 => Num9,
			29 => Num0,

			// Symbols
			50 => Grave,        // ~ `
			33 => LeftBracket,  // { [
			30 => RightBracket, // } ]
			42 => Backslash,    // | \
			41 => Semicolon,    // : ;
			39 => Quote,        // " '
			27 => Minus,        // _ -
			24 => Equal,        // + =
			43 => Comma,        // < ,
			47 => Period,       // > .
			44 => Slash,        // ? /
			10 => Section,      // § (EU/ISO keyboards only)

			// Numpad
			82 => Numpad0,
			83 => Numpad1,
			84 => Numpad2,
			85 => Numpad3,
			86 => Numpad4,
			87 => Numpad5,
			88 => Numpad6,
			89 => Numpad7,
			91 => Numpad8,
			92 => Numpad9,
			67 => NumpadMultiply,
			69 => NumpadAdd,
			78 => NumpadSubtract,
			75 => NumpadDivide,
			76 => NumpadEnter,
			65 => NumpadDecimal,
			71 => NumpadClear,
			95 => NumpadComma,
			72 => VolumeUp,
			73 => VolumeDown,
			74 => Mute,
			93 => Yen,
			94 => Underscore,
			102 => Eisu,
			104 => Kana,
			110 => Menu,
			114 => Help,
			117 => ForwardDelete,
			160 => MissionControl,
			127 => Power,

			52 => EnterPowerbook,

			66 | 68 | 70 | 77 | 81 | 108 | 112 => return Err(()), // Haven't found out what these are yet

			128..=u16::MAX => return Err(()),
		})
	}
}

impl From<Key> for u16 {
	fn from(key: Key) -> Self {
		key as u16
	}
}

impl From<&Key> for u16 {
	fn from(key: &Key) -> Self {
		*key as u16
	}
}

impl TryFrom<Key> for char {
	type Error = ();
	fn try_from(value: Key) -> Result<char, Self::Error> {
		use Key::*;
		Ok(match value {
			A => 'a',
			B => 'b',
			C => 'c',
			D => 'd',
			E => 'e',
			F => 'f',
			G => 'g',
			H => 'h',
			I => 'i',
			J => 'j',
			K => 'k',
			L => 'l',
			M => 'm',
			N => 'n',
			O => 'o',
			P => 'p',
			Q => 'q',
			R => 'r',
			S => 's',
			T => 't',
			U => 'u',
			V => 'v',
			W => 'w',
			X => 'x',
			Y => 'y',
			Z => 'z',
			Num0 => '0',
			Num1 => '1',
			Num2 => '2',
			Num3 => '3',
			Num4 => '4',
			Num5 => '5',
			Num6 => '6',
			Num7 => '7',
			Num8 => '8',
			Num9 => '9',
			Section => '§',
			Grave => '`',
			LeftBracket => '[',
			RightBracket => ']',
			Backslash => '\\',
			Semicolon => ';',
			Quote => '\'',
			Minus => '-',
			Equal => '=',
			Comma => ',',
			Period => '.',
			Slash => '/',
			_ => return Err(()),
		})
	}
}

impl TryFrom<&Key> for char {
	type Error = ();
	fn try_from(value: &Key) -> Result<char, Self::Error> {
		char::try_from(*value)
	}
}

#[cfg(feature = "serde")]
impl std::str::FromStr for Key {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.is_empty() {
			return Err("empty key name".into());
		}
		serde_json::from_value(serde_json::Value::String(s.into())).map_err(|_| format!("unknown key: {s}"))
	}
}

impl Key {
	pub fn down(self, modifiers: Modifiers, location: EventLocation) -> Result<(), Error> {
		let (event, location) = new_keyboard_event(self, modifiers, location, true)?;
		//                                       only real difference to "up" ^^^^ (is_down)

		location.post(&event)?;

		Ok(())
	}

	pub fn up(self, modifiers: Modifiers, location: EventLocation) -> Result<(), Error> {
		let (event, location) = new_keyboard_event(self, modifiers, location, false)?;
		//                                     only real difference to "down" ^^^^^ (is_down)

		location.post(&event)?;

		Ok(())
	}
}

/// Gets the preferred [EventLocation] for a given keyboard key and PID.
///
/// This is only intended to transform `EventLocation::Process` to `EventLocation::Hid` (as indicated by the fact that there is a `pid` argument. HID location doesn't take a PID.)
///
/// Useful, because sometimes you may want to prefer posting the event to the HID location instead of to an application/process directly,
/// but only if this wouldn't mess up keyboard layout/locale.
///
/// Take this example, which is why I implemented this function:
///
/// 1. A custom slip function for Ableton Live simulates pressing Cmd+Option+Shift and then pressing down `MouseButton::Left`.
/// 2. Mouse events need to be posted to the HID location (otherwise macOS acts up)
/// 3. If we don't post keyboard events to the same location (HID) as mouse events, there's a race condition where
///    the mouse event will be processed BEFORE the keyboard event. The two queues seem to be processed independently as of writing (macOS Tahoe 26.1).
/// 4. Keyboard events need to respect locale. So we explicitly set the unicode string for each event (`CGEvent::set_unicode_string`),
///    which ensures that simulating pressing `=` really results in a `=` being posted. But this only works at the Process location.
///
///    Small tangent:
///    On other keyboards (e.g. German DE, Swedish SE, ...), the place where the `=` key is is different from the US layout.
///    But the keycode emitted is the same. This is how keyboards work. The key in the top right, where on my U.S. keyboard there's a `=`,
///    will always send 0x18 to macOS. And German, Swedish, Japanese keyboards will have a different character printed on them,
///    but still send 0x18. Because the OS is supposed to do the translation.
///
///    The point: Posting a unicode string, so simulated keypresses are the same no matter the keyboard layout,
///    ***only works at the Process location***
///
/// 5. All of the keyboards have SOME equivalent keys. For example, the Cmd key is always in the same place and always code 0x37. (Purely by convention, I think.)
/// 6. For these keys, why not prefer posting to the HID location? So there's no race condition between mouse and keyboard?
/// 7. But we can't simply decide to override `EventLocation::Process` if that was passed to `Key::down()`.
///    What if someone wants an event only to be seen by the target process, and hidden from other apps?
///    (Not as a security measure, but to prevent them from acting on the received keypress)
/// 8. So we'll have a function -- preferred_event_location -- that callers can use to let us decide when to post to HID instead of Process,
///    based on whether the key is a printable character (which is likely to be locale-dependent) or not. If it's a printable character,
///    it HAS to be posted to `::Process`, because otherwise the keycode may get translated by macOS.
///    But, for example, for simulating only modifier keys being pressed down, we can post to HID instead. There shouldn't be any translation.
pub fn preferred_event_location(key: &Key, pid: u32) -> EventLocation {
	if char::try_from(key).is_ok() {
		EventLocation::Process { pid }
	} else {
		EventLocation::Hid
	}
}

fn new_keyboard_event(key: Key, modifiers: Modifiers, location: EventLocation, key_down: bool) -> Result<(CFRetained<CGEvent>, EventLocation), Error> {
	let flags = CGEventFlags::from(modifiers);

	// 1. Create event
	let event = CGEvent::new_keyboard_event(None, key.into(), key_down).ok_or(Error::EventCreationFailed)?;

	// 2. Set modifiers ("flags")
	CGEvent::set_flags(Some(&event), flags);

	// 3. Try to set unicode string instead of raw keycode.
	//    This helps with keyboard localization.
	//    Note: this doesn't work for `EventLocation::Hid`.
	if let Ok(uni_char) = char::try_from(key) {
		unsafe { CGEvent::keyboard_set_unicode_string(Some(&event), 1, &(uni_char as u16)) };
	}

	Ok((event, location))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
pub struct Modifiers(u16);

bitflags::bitflags! {
	impl Modifiers: u16 {
		// IMPORTANT: If you change the name of a modifier, you must also change the corresponding `modifier_name` below.
		// Otherwise stuff like `KeySet::from(Modifiers)` will break.
		const CapsLock = 1 << 1;        // MaskAlphaShift
		const Shift = 1 << 2;            // MaskShift
		const Control = 1 << 3;          // MaskControl
		const Option = 1 << 4;           // MaskAlternate
		const Command = 1 << 5;          // MaskCommand
		const Help = 1 << 6;             // MaskHelp
		const SecondaryFn = 1 << 7;     // MaskSecondaryFn
		const Numpad = 1 << 8;           // MaskNumericPad
		// const NonCoalesced = 1 << 9;    // MaskNonCoalesced
	}
}

impl TryFrom<Key> for Modifiers {
	type Error = ();

	fn try_from(key: Key) -> std::result::Result<Self, ()> {
		match key {
			Key::LeftShift | Key::RightShift => Ok(Self::Shift),
			Key::LeftControl | Key::RightControl => Ok(Self::Control),
			Key::LeftOption | Key::RightOption => Ok(Self::Option),
			Key::LeftCommand | Key::RightCommand => Ok(Self::Command),
			Key::Globe => Ok(Self::SecondaryFn),
			Key::CapsLock => Ok(Self::CapsLock),
			_ => Err(()),
		}
	}
}

mod modifier_name {
	pub const CAPS_LOCK: &str = "CapsLock";
	pub const SHIFT: &str = "Shift";
	pub const CONTROL: &str = "Control";
	pub const OPTION: &str = "Option";
	pub const COMMAND: &str = "Command";
	pub const HELP: &str = "Help";
	pub const SECONDARY_FN: &str = "SecondaryFn";
	pub const NUMPAD: &str = "Numpad";
}

impl From<CGEventFlags> for Modifiers {
	fn from(flags: CGEventFlags) -> Self {
		let mut modifiers = Self::empty();

		if flags.contains(CGEventFlags::MaskShift) {
			modifiers.insert(Modifiers::Shift);
		}
		if flags.contains(CGEventFlags::MaskControl) {
			modifiers.insert(Modifiers::Control);
		}
		if flags.contains(CGEventFlags::MaskAlternate) {
			modifiers.insert(Modifiers::Option);
		}
		if flags.contains(CGEventFlags::MaskCommand) {
			modifiers.insert(Modifiers::Command);
		}
		if flags.contains(CGEventFlags::MaskHelp) {
			modifiers.insert(Modifiers::Help);
		}
		if flags.contains(CGEventFlags::MaskSecondaryFn) {
			modifiers.insert(Modifiers::SecondaryFn);
		}
		if flags.contains(CGEventFlags::MaskNumericPad) {
			modifiers.insert(Modifiers::Numpad);
		}
		if flags.contains(CGEventFlags::MaskAlphaShift) {
			modifiers.insert(Modifiers::CapsLock);
		}

		modifiers
	}
}

impl From<Modifiers> for CGEventFlags {
	fn from(modifiers: Modifiers) -> Self {
		let mut flags = CGEventFlags::empty();

		if modifiers.contains(Modifiers::Shift) {
			flags.insert(CGEventFlags::MaskShift);
		}
		if modifiers.contains(Modifiers::Control) {
			flags.insert(CGEventFlags::MaskControl);
		}
		if modifiers.contains(Modifiers::Option) {
			flags.insert(CGEventFlags::MaskAlternate);
		}
		if modifiers.contains(Modifiers::Command) {
			flags.insert(CGEventFlags::MaskCommand);
		}
		if modifiers.contains(Modifiers::Help) {
			flags.insert(CGEventFlags::MaskHelp);
		}
		if modifiers.contains(Modifiers::SecondaryFn) {
			flags.insert(CGEventFlags::MaskSecondaryFn);
		}
		if modifiers.contains(Modifiers::Numpad) {
			flags.insert(CGEventFlags::MaskNumericPad);
		}
		if modifiers.contains(Modifiers::CapsLock) {
			flags.insert(CGEventFlags::MaskAlphaShift);
		}

		flags
	}
}

impl std::str::FromStr for Modifiers {
	type Err = String;

	/// Parse modifier(s) from a `+`-separated string. Each part is resolved
	/// as an alias ("cmd"), a canonical name ("Command"), or a modifier key
	/// name ("leftCommand"). Case-insensitive for aliases.
	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let mut mods = Self::empty();
		for part in s.split('+') {
			let key = part.parse::<Key>().map_err(|_| format!("unknown modifier: {part}"))?;
			mods |= Modifiers::try_from(key).map_err(|_| format!("not a modifier: {part}"))?;
		}
		if mods.is_empty() {
			return Err("empty modifiers".into());
		}
		Ok(mods)
	}
}

/// A key combo: optional main key + modifier flags. Parses from strings like "cmd+shift+a".
/// Modifiers must come before the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Accelerator {
	pub key: Option<Key>,
	pub modifiers: Modifiers,
}

impl Accelerator {
	pub fn new(key: Option<Key>, modifiers: Modifiers) -> Self {
		Self { key, modifiers }
	}

	/// Resolve the preferred event location for this accelerator's key.
	pub fn location(&self, pid: u32) -> EventLocation {
		match self.key {
			Some(key) => preferred_event_location(&key, pid),
			None => EventLocation::Hid,
		}
	}

	pub fn press(self, location: EventLocation) -> Result<(), Error> {
		self.down(location)?;
		self.up(location)
	}

	pub fn down(self, location: EventLocation) -> Result<(), Error> {
		if let Some(key) = self.key {
			key.down(self.modifiers, location)
		} else {
			Ok(())
		}
	}

	pub fn up(self, location: EventLocation) -> Result<(), Error> {
		if let Some(key) = self.key { key.up(self.modifiers, location) } else { Ok(()) }
	}
}

#[cfg(feature = "serde")]
impl std::str::FromStr for Accelerator {
	type Err = String;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		if s.is_empty() {
			return Err("empty accelerator".into());
		}

		let mut modifiers = Modifiers::empty();
		let mut main_key = None;

		for part in s.split('+') {
			let key = part.parse::<Key>().map_err(|_| format!("unknown key: {part}"))?;
			if let Ok(m) = Modifiers::try_from(key) {
				if main_key.is_some() {
					return Err(format!("modifiers must come before key: {s}"));
				}
				modifiers |= m;
			} else {
				if main_key.is_some() {
					return Err(format!("multiple non-modifier keys: {s}"));
				}
				main_key = Some(key);
			}
		}

		if main_key.is_none() && modifiers.is_empty() {
			return Err(format!("empty accelerator: {s}"));
		}

		Ok(Accelerator { key: main_key, modifiers })
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(into = "Vec<Key>", try_from = "KeySetInput")
)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema), schemars(with = "Vec<Key>"))]
pub struct KeySet(#[cfg_attr(feature = "specta", specta(type = Vec<Key>))] EnumSet<Key>);

/// Accepts either `["leftCommand", "a"]` or `"cmd+a"`.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(untagged)]
enum KeySetInput {
	Parts(Vec<Key>),
	Combo(String),
}

#[cfg(feature = "serde")]
impl TryFrom<KeySetInput> for KeySet {
	type Error = String;

	fn try_from(input: KeySetInput) -> Result<Self, Self::Error> {
		match input {
			KeySetInput::Parts(keys) => Ok(KeySet::new(keys)),
			KeySetInput::Combo(s) => s.parse(),
		}
	}
}

#[cfg(feature = "serde")]
impl std::str::FromStr for KeySet {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let keys: Result<Vec<Key>, _> = s.split('+').map(|p| p.parse::<Key>()).collect();
		Ok(KeySet::new(keys?))
	}
}

impl From<KeySet> for Vec<Key> {
	fn from(set: KeySet) -> Self {
		set.0.iter().collect()
	}
}

impl From<Vec<Key>> for KeySet {
	fn from(value: Vec<Key>) -> Self {
		Self::new(value)
	}
}

impl KeySet {
	pub fn new(keys: impl IntoIterator<Item = Key>) -> Self {
		Self(EnumSet::from_iter(keys))
	}

	pub fn empty() -> Self {
		Self(EnumSet::empty())
	}

	pub fn single(key: Key) -> Self {
		Self(EnumSet::only(key))
	}

	pub fn single_with_modifiers(key: Key, modifiers: Modifiers) -> Self {
		Self(KeySet::single(key).0 | KeySet::from(modifiers).0)
	}

	pub fn contains_key(&self, key: Key) -> bool {
		self.0.contains(key)
	}

	pub fn intersects(&self, other: &Self) -> bool {
		!self.0.intersection(other.0).is_empty()
	}
}

impl From<EnumSet<Key>> for KeySet {
	fn from(set: EnumSet<Key>) -> Self {
		Self(set)
	}
}

impl From<Key> for KeySet {
	fn from(key: Key) -> Self {
		Self::single(key)
	}
}

impl From<Modifiers> for KeySet {
	fn from(modifiers: Modifiers) -> Self {
		KeySet::new(modifiers.iter_names().filter_map(|(name, _)| match name {
			modifier_name::CAPS_LOCK => Some(Key::CapsLock),
			modifier_name::SHIFT => Some(Key::LeftShift),
			modifier_name::CONTROL => Some(Key::LeftControl),
			modifier_name::OPTION => Some(Key::LeftOption),
			modifier_name::COMMAND => Some(Key::LeftCommand),
			modifier_name::HELP => Some(Key::Help),
			modifier_name::SECONDARY_FN => Some(Key::Globe), // TODO is this correct?
			modifier_name::NUMPAD => None, // TODO is this a bug? technically the Numpad modifier should modify the OTHER keys, e.g. the first argument of single_with_modifiers, no? Num4 -> Numpad4, for example
			_ => None,
		}))
	}
}

impl From<KeySet> for EnumSet<Key> {
	fn from(set: KeySet) -> Self {
		set.0
	}
}

impl Debug for KeySet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "KeySet({:?})", self.0)
	}
}

#[cfg(feature = "napi")]
impl napi::bindgen_prelude::ToNapiValue for KeySet {
	unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> napi::Result<napi::sys::napi_value> {
		use napi::bindgen_prelude::*;
		unsafe {
			// Always return array for consistency with serde representation
			let keys: Vec<Key> = val.into();
			ToNapiValue::to_napi_value(env, keys)
		}
	}
}

#[cfg(feature = "napi")]
impl napi::bindgen_prelude::FromNapiValue for KeySet {
	unsafe fn from_napi_value(env: napi::sys::napi_env, napi_val: napi::sys::napi_value) -> napi::Result<Self> {
		unsafe {
			let keys: Vec<Key> = napi::bindgen_prelude::FromNapiValue::from_napi_value(env, napi_val)?;
			Ok(keys.into())
		}
	}
}

#[cfg(all(test, feature = "serde"))]
mod tests {
	use super::*;

	mod key_from_str {
		use super::*;

		#[test]
		fn camel_case() {
			assert_eq!("leftCommand".parse::<Key>().unwrap(), Key::LeftCommand);
			assert_eq!("leftShift".parse::<Key>().unwrap(), Key::LeftShift);
			assert_eq!("leftOption".parse::<Key>().unwrap(), Key::LeftOption);
			assert_eq!("rightCommand".parse::<Key>().unwrap(), Key::RightCommand);
		}

		#[test]
		fn aliases() {
			assert_eq!("cmd".parse::<Key>().unwrap(), Key::LeftCommand);
			assert_eq!("command".parse::<Key>().unwrap(), Key::LeftCommand);
			assert_eq!("ctrl".parse::<Key>().unwrap(), Key::LeftControl);
			assert_eq!("control".parse::<Key>().unwrap(), Key::LeftControl);
			assert_eq!("opt".parse::<Key>().unwrap(), Key::LeftOption);
			assert_eq!("option".parse::<Key>().unwrap(), Key::LeftOption);
			assert_eq!("alt".parse::<Key>().unwrap(), Key::LeftOption);
			assert_eq!("shift".parse::<Key>().unwrap(), Key::LeftShift);
			assert_eq!("esc".parse::<Key>().unwrap(), Key::Escape);
			assert_eq!("fn".parse::<Key>().unwrap(), Key::Globe);
		}

		#[test]
		fn letters() {
			assert_eq!("a".parse::<Key>().unwrap(), Key::A);
			assert_eq!("z".parse::<Key>().unwrap(), Key::Z);
		}

		#[test]
		fn numbers() {
			assert_eq!("1".parse::<Key>().unwrap(), Key::Num1);
			assert_eq!("0".parse::<Key>().unwrap(), Key::Num0);
			assert_eq!("num5".parse::<Key>().unwrap(), Key::Num5);
		}

		#[test]
		fn symbols() {
			assert_eq!("[".parse::<Key>().unwrap(), Key::LeftBracket);
			assert_eq!("]".parse::<Key>().unwrap(), Key::RightBracket);
			assert_eq!("-".parse::<Key>().unwrap(), Key::Minus);
			assert_eq!("=".parse::<Key>().unwrap(), Key::Equal);
			assert_eq!(",".parse::<Key>().unwrap(), Key::Comma);
			assert_eq!(".".parse::<Key>().unwrap(), Key::Period);
			assert_eq!("/".parse::<Key>().unwrap(), Key::Slash);
			assert_eq!(";".parse::<Key>().unwrap(), Key::Semicolon);
		}

		#[test]
		fn arrows() {
			assert_eq!("left".parse::<Key>().unwrap(), Key::LeftArrow);
			assert_eq!("right".parse::<Key>().unwrap(), Key::RightArrow);
			assert_eq!("up".parse::<Key>().unwrap(), Key::UpArrow);
			assert_eq!("down".parse::<Key>().unwrap(), Key::DownArrow);
			assert_eq!("upArrow".parse::<Key>().unwrap(), Key::UpArrow);
			assert_eq!("downArrow".parse::<Key>().unwrap(), Key::DownArrow);
		}

		#[test]
		fn rejects_empty() {
			assert!("".parse::<Key>().is_err());
		}

		#[test]
		fn rejects_unknown() {
			assert!("notAKey".parse::<Key>().is_err());
			assert!("Cmd".parse::<Key>().is_err()); // case-sensitive: serde uses camelCase
		}
	}

	mod key_set_from_str {
		use super::*;

		#[test]
		fn single_key() {
			let ks = "a".parse::<KeySet>().unwrap();
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn combo_camel_case() {
			let ks = "leftCommand+a".parse::<KeySet>().unwrap();
			assert!(ks.contains_key(Key::LeftCommand));
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn combo_aliases() {
			let ks = "cmd+shift+a".parse::<KeySet>().unwrap();
			assert!(ks.contains_key(Key::LeftCommand));
			assert!(ks.contains_key(Key::LeftShift));
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn trailing_plus_errors() {
			assert!("cmd+".parse::<KeySet>().is_err());
		}

		#[test]
		fn leading_plus_errors() {
			assert!("+cmd".parse::<KeySet>().is_err());
		}

		#[test]
		fn double_plus_errors() {
			assert!("cmd++a".parse::<KeySet>().is_err());
		}

		#[test]
		fn empty_errors() {
			assert!("".parse::<KeySet>().is_err());
		}
	}

	mod key_set_deserialize {
		use super::*;

		#[test]
		fn from_array() {
			let ks: KeySet = serde_json::from_str(r#"["leftCommand", "a"]"#).unwrap();
			assert!(ks.contains_key(Key::LeftCommand));
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn from_array_with_aliases() {
			let ks: KeySet = serde_json::from_str(r#"["cmd", "shift", "a"]"#).unwrap();
			assert!(ks.contains_key(Key::LeftCommand));
			assert!(ks.contains_key(Key::LeftShift));
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn from_string_combo() {
			let ks: KeySet = serde_json::from_str(r#""cmd+a""#).unwrap();
			assert!(ks.contains_key(Key::LeftCommand));
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn from_string_combo_canonical() {
			let ks: KeySet = serde_json::from_str(r#""leftCommand+leftShift+a""#).unwrap();
			assert!(ks.contains_key(Key::LeftCommand));
			assert!(ks.contains_key(Key::LeftShift));
			assert!(ks.contains_key(Key::A));
		}

		#[test]
		fn string_trailing_plus_errors() {
			assert!(serde_json::from_str::<KeySet>(r#""cmd+""#).is_err());
		}

		#[test]
		fn string_empty_errors() {
			assert!(serde_json::from_str::<KeySet>(r#""""#).is_err());
		}

		#[test]
		fn array_and_string_are_equivalent() {
			let from_arr: KeySet = serde_json::from_str(r#"["leftCommand", "leftOption", "b"]"#).unwrap();
			let from_str: KeySet = serde_json::from_str(r#""cmd+opt+b""#).unwrap();
			assert_eq!(from_arr, from_str);
		}

		#[test]
		fn serializes_as_array() {
			let ks: KeySet = serde_json::from_str(r#""cmd+a""#).unwrap();
			let json = serde_json::to_value(&ks).unwrap();
			assert!(json.is_array());
		}
	}

	mod accelerator {
		use super::*;

		#[test]
		fn simple_combo() {
			let acc = "cmd+a".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::A));
			assert_eq!(acc.modifiers, Modifiers::Command);
		}

		#[test]
		fn multiple_modifiers() {
			let acc = "cmd+shift+e".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::E));
			assert_eq!(acc.modifiers, Modifiers::Command | Modifiers::Shift);
		}

		#[test]
		fn canonical_key_names() {
			let acc = "leftCommand+leftShift+a".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::A));
			assert_eq!(acc.modifiers, Modifiers::Command | Modifiers::Shift);
		}

		#[test]
		fn mixed_aliases_and_canonical() {
			let acc = "cmd+leftShift+b".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::B));
			assert_eq!(acc.modifiers, Modifiers::Command | Modifiers::Shift);
		}

		#[test]
		fn all_modifier_aliases() {
			let acc = "cmd+shift+ctrl+opt+fn+caps+a".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::A));
			assert!(acc.modifiers.contains(Modifiers::Command));
			assert!(acc.modifiers.contains(Modifiers::Shift));
			assert!(acc.modifiers.contains(Modifiers::Control));
			assert!(acc.modifiers.contains(Modifiers::Option));
			assert!(acc.modifiers.contains(Modifiers::SecondaryFn));
			assert!(acc.modifiers.contains(Modifiers::CapsLock));
		}

		#[test]
		fn alt_aliases() {
			let a1 = "alt+a".parse::<Accelerator>().unwrap();
			let a2 = "option+a".parse::<Accelerator>().unwrap();
			let a3 = "opt+a".parse::<Accelerator>().unwrap();
			assert_eq!(a1, a2);
			assert_eq!(a2, a3);
			assert_eq!(a1.modifiers, Modifiers::Option);
		}

		#[test]
		fn key_only_no_modifiers() {
			let acc = "a".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::A));
			assert_eq!(acc.modifiers, Modifiers::empty());
		}

		#[test]
		fn function_key() {
			let acc = "cmd+f1".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::F1));
			assert_eq!(acc.modifiers, Modifiers::Command);
		}

		#[test]
		fn right_modifier_keys() {
			let acc = "rightCommand+rightShift+a".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, Some(Key::A));
			assert!(acc.modifiers.contains(Modifiers::Command));
			assert!(acc.modifiers.contains(Modifiers::Shift));
		}

		#[test]
		fn alias_and_canonical_equivalent() {
			let a1 = "cmd+a".parse::<Accelerator>().unwrap();
			let a2 = "leftCommand+a".parse::<Accelerator>().unwrap();
			assert_eq!(a1, a2);
		}

		#[test]
		fn multiple_non_modifier_keys_errors() {
			assert!("a+b".parse::<Accelerator>().is_err());
		}

		#[test]
		fn trailing_plus_errors() {
			assert!("cmd+".parse::<Accelerator>().is_err());
		}

		#[test]
		fn empty_errors() {
			assert!("".parse::<Accelerator>().is_err());
		}

		#[test]
		fn modifiers_only() {
			let acc = "cmd+shift".parse::<Accelerator>().unwrap();
			assert_eq!(acc.key, None);
			assert_eq!(acc.modifiers, Modifiers::Command | Modifiers::Shift);
		}

		#[test]
		fn unknown_key_errors() {
			assert!("cmd+notakey".parse::<Accelerator>().is_err());
		}
	}
}
