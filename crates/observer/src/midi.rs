//! CoreMIDI input observer.
//!
//! A [`MidiObserver`] listens to CoreMIDI sources and calls its callback once
//! per logical MIDI UMP message. CoreMIDI may deliver an [`EventList`] as
//! packets that contain one or more messages; that packet shape is transport
//! detail, not the event boundary this module exposes.
//!
//! The observer's opinion is small: a MIDI event has a source id, a timestamp,
//! and a parsed [`midi2`] message. It does not map MIDI into shortcut triggers,
//! app commands, or any other product-specific vocabulary. Consumers own that
//! projection.
//!
//! This is deliberately shaped like a small "MidiKit" API, not an Invoke API.
//! If another app used this crate, it should still make sense: observe messages,
//! decide whether a message was claimed, and optionally route unclaimed messages
//! onward. Shortcut semantics live above this crate.
//!
//! Why `coremidi` + [`midi2`]:
//!
//! - `coremidi` owns macOS transport and source connections.
//! - [`midi2`] owns MIDI message parsing and message-family types.
//! - This crate should not maintain pointer walking, bit shifting, or a local
//!   copy of the MIDI protocol.
//!
//! Why expose [`midi2`]:
//!
//! MIDI is already a rich protocol surface. Recreating every message family in
//! this crate would add an opinion we do not currently need. The observer's
//! opinion is the boundary: one event per message, with source and timestamp.
//!
//! Why return [`CallbackResult`]:
//!
//! Eventtap lets consumers decide whether an observed event should continue
//! downstream. When a [`MidiThru`] is attached, the same pass/drop disposition
//! decides whether the original MIDI message is forwarded to the virtual source.
//! Without a Thru, the observer only observes.
//!
//! Why Thru lives beside the observer:
//!
//! Forwarding has to use the original UMP words, not a parse-and-reserialize
//! copy. That is how SysEx, MIDI-CI, future/proprietary messages, and parser
//! edge cases keep working. Those words are a transport detail, though, so they
//! stay inside this module instead of becoming public API. The public surface is
//! still message-level: callback gets an [`Event`], callback returns
//! [`CallbackResult`].
//!
//! If Thru later becomes per-device, keep that same boundary. The thing to vary
//! is the internal route chosen for a source/message, not the shape of [`Event`]
//! and not a public "raw words" escape hatch.

use crate::{Observer, ObserverGuard};
use coremidi::{Client, EventBuffer, EventList, InputPortWithContext, Protocol, Source, Sources, VirtualSource};
pub use midi2;
use midi2::{Data, RebufferInto};
use std::sync::Mutex;
use thiserror::Error;

/// A parsed, owned MIDI UMP message.
pub type Message = midi2::UmpMessage<Vec<u32>>;

type OnEvent = dyn Fn(Event) -> CallbackResult + Send + Sync + 'static;

const PROTOCOL: Protocol = Protocol::Midi10;

#[derive(Debug, Error)]
pub enum Error {
	#[error("CoreMIDI returned OSStatus {0}")]
	CoreMidi(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackResult {
	/// The consumer did not claim the message. A downstream route may forward it.
	Passthrough,

	/// The consumer claimed the message. A downstream route should suppress it.
	Drop,
}

/// One observed MIDI message.
///
/// This is intentionally not a CoreMIDI packet and not an app-level trigger.
/// A single CoreMIDI packet may produce multiple [`Event`] values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
	/// CoreMIDI's stable source endpoint identity.
	pub source_id: i32,

	/// CoreMIDI packet timestamp for the packet this message came from.
	pub timestamp: u64,

	/// The parsed MIDI message.
	pub message: Message,
}

/// Optional virtual MIDI source for byte-exact message forwarding.
///
/// Consumers should not receive raw UMP words through the public observer API.
/// Thru forwarding is colocated here so the receive path can use the private
/// original message words without making them a supported public contract.
///
/// This is a CoreMIDI virtual source, not global MIDI interception. A downstream
/// app only becomes suppressible if it listens to this source instead of the
/// original hardware source.
#[derive(Debug)]
pub struct MidiThru {
	_client: Client,
	source: VirtualSource,
	source_id: i32,
}

impl MidiThru {
	pub fn new(name: &str) -> Result<Self, Error> {
		let client = Client::new(name).map_err(Error::CoreMidi)?;
		let source = client.virtual_source(name).map_err(Error::CoreMidi)?;
		let source_id = source.get_property(&coremidi::Properties::unique_id()).map_err(Error::CoreMidi)?;

		Ok(Self {
			_client: client,
			source,
			source_id,
		})
	}
}

#[derive(Debug)]
struct SourceContext {
	source_id: i32,
}

#[derive(Debug)]
pub struct MidiObserver {
	_client: Client,
	state: Mutex<State>,
}

#[derive(Debug)]
struct State {
	input: InputPortWithContext<SourceContext>,
	sources: Vec<Source>,
	/// CoreMIDI exposes our virtual source in the same source list as hardware
	/// endpoints. Do not connect the observer back to its own Thru source.
	thru_source_id: Option<i32>,
}

impl MidiObserver {
	pub fn new(on_event: impl Fn(Event) -> CallbackResult + Send + Sync + 'static) -> Result<ObserverGuard<Self>, Error> {
		Self::new_inner(Box::new(on_event), None)
	}

	pub fn new_with_thru(on_event: impl Fn(Event) -> CallbackResult + Send + Sync + 'static, thru: MidiThru) -> Result<ObserverGuard<Self>, Error> {
		Self::new_inner(Box::new(on_event), Some(thru))
	}

	fn new_inner(on_event: Box<OnEvent>, thru: Option<MidiThru>) -> Result<ObserverGuard<Self>, Error> {
		let client = Client::new("MIDI Observer").map_err(Error::CoreMidi)?;
		let thru_source_id = thru.as_ref().map(|thru| thru.source_id);

		let input = client
			.input_port_with_protocol("MIDI Input", PROTOCOL, move |event_list, source: &mut SourceContext| {
				handle_event_list(event_list, source.source_id, on_event.as_ref(), |timestamp, words| {
					let Some(thru) = &thru else {
						return;
					};

					let buffer = EventBuffer::new(PROTOCOL).with_packet(timestamp, words);

					if let Err(status) = thru.source.received(buffer) {
						log::warn!("Failed to forward MIDI message through source {}: OSStatus {status}", thru.source_id);
					}
				})
			})
			.map_err(Error::CoreMidi)?;

		Ok(ObserverGuard {
			inner: Self {
				_client: client,
				state: Mutex::new(State {
					input,
					sources: Vec::new(),
					thru_source_id,
				}),
			},
		})
	}
}

impl Observer for MidiObserver {
	type Error = Error;
	type Event = Event;

	// This means "make sure we're connected to the current MIDI sources".
	// Calling it twice must not connect to the same source twice.
	fn listen(&self) -> Result<(), Self::Error> {
		let mut state = self.state.lock().unwrap();

		for source in Sources {
			let source_id = source_id(&source)?;

			// Without this, a downstream app or virtual bus can feed our own
			// forwarded messages back into the observer and create duplicates or
			// feedback loops. If Thru becomes per-device, this becomes a set of
			// Invoke-owned source ids.
			if Some(source_id) == state.thru_source_id {
				continue;
			}

			if state.sources.iter().any(|connected| connected == &source) {
				continue;
			}

			state.input.connect_source(&source, SourceContext { source_id }).map_err(Error::CoreMidi)?;
			state.sources.push(source);
		}

		Ok(())
	}

	fn sleep(&self) -> Result<(), Self::Error> {
		let mut state = self.state.lock().unwrap();
		let sources = std::mem::take(&mut state.sources);

		for source in sources {
			if let Err(status) = state.input.disconnect_source(&source) {
				log::warn!("Failed to disconnect MIDI source: OSStatus {status}");
			}
		}

		Ok(())
	}
}

fn source_id(source: &Source) -> Result<i32, Error> {
	source.get_property(&coremidi::Properties::unique_id()).map_err(Error::CoreMidi)
}

fn handle_event_list(event_list: &EventList, source_id: i32, on_event: &dyn Fn(Event) -> CallbackResult, mut forward: impl FnMut(u64, &[u32])) {
	for packet in event_list {
		handle_packet(packet.timestamp(), packet.data(), source_id, on_event, &mut forward);
	}
}

/// Route one CoreMIDI packet at MIDI-message granularity.
///
/// CoreMIDI packets can contain multiple UMP messages. The callback must run
/// once per message so a claimed note/control/program change can be dropped
/// without dropping unrelated messages that happened to arrive in the same
/// packet.
///
/// Parsing is for the public [`Event`]. Forwarding uses `message_words`, the
/// exact original slice from the packet. If parsing fails, there is no safe
/// message-level decision to ask the callback for, so Thru mode forwards the
/// remaining bytes unchanged and stops trying to split that packet.
fn handle_packet(timestamp: u64, mut words: &[u32], source_id: i32, on_event: &dyn Fn(Event) -> CallbackResult, forward: &mut impl FnMut(u64, &[u32])) {
	while !words.is_empty() {
		let Some((message, consumed)) = next_message(words) else {
			forward(timestamp, words);
			break;
		};

		let message_words = &words[..consumed];
		let result = on_event(Event {
			source_id,
			timestamp,
			message: message.rebuffer_into(),
		});

		if result == CallbackResult::Passthrough {
			forward(timestamp, message_words);
		}

		words = &words[consumed..];
	}
}

/// Parse one MIDI message from the front of a CoreMIDI packet's UMP words.
///
/// `midi2` validates the message and tells us how many words belong to it via
/// [`Data::data`]. This keeps protocol framing out of this crate.
fn next_message(words: &[u32]) -> Option<(midi2::UmpMessage<&[u32]>, usize)> {
	let message = midi2::UmpMessage::try_from(words).ok()?;
	let consumed = message.data().len();
	Some((message, consumed))
}

#[cfg(test)]
mod tests {
	use super::*;
	use midi2::prelude::*;
	use std::cell::Cell;

	#[test]
	fn parses_note_on() {
		let event = event_from_words(42, 100, &midi1_note_on(64));

		assert_eq!(event.source_id, 42);
		assert_eq!(event.timestamp, 100);
		assert!(matches!(event.message, midi2::UmpMessage::ChannelVoice1(_)));
	}

	#[test]
	fn note_on_velocity_zero_stays_note_on() {
		use midi2::channel_voice1::ChannelVoice1::*;

		let event = event_from_words(42, 100, &midi1_note_on(0));

		let midi2::UmpMessage::ChannelVoice1(NoteOn(message)) = event.message else {
			panic!("expected note on");
		};
		assert_eq!(u8::from(message.velocity()), 0);
	}

	#[test]
	fn parser_consumes_back_to_back_messages() {
		let words = [midi1_note_on(64), midi1_note_off()].concat();
		let messages = parsed_messages(&words);

		assert_eq!(messages.len(), 2);
	}

	#[test]
	fn parser_consumes_two_word_messages() {
		let words = [midi2_note_on(), midi1_note_on(64)].concat();
		let (message, consumed) = next_message(&words).unwrap();

		assert_eq!(consumed, 2);
		assert!(matches!(message, midi2::UmpMessage::ChannelVoice2(_)));
	}

	#[test]
	fn passthrough_forwards_original_words() {
		let words = midi1_note_on(64);
		let mut forwarded = Vec::new();
		let mut forward = |timestamp, words: &[u32]| forwarded.push((timestamp, words.to_vec()));
		let on_event = |_| CallbackResult::Passthrough;

		handle_packet(100, &words, 42, &on_event, &mut forward);

		assert_eq!(forwarded, vec![(100, words)]);
	}

	#[test]
	fn drop_does_not_forward() {
		let words = midi1_note_on(64);
		let mut forwarded = Vec::new();
		let mut forward = |timestamp, words: &[u32]| forwarded.push((timestamp, words.to_vec()));
		let on_event = |_| CallbackResult::Drop;

		handle_packet(100, &words, 42, &on_event, &mut forward);

		assert!(forwarded.is_empty());
	}

	#[test]
	fn back_to_back_messages_route_independently() {
		let note_on = midi1_note_on(64);
		let note_off = midi1_note_off();
		let words = [note_on, note_off.clone()].concat();
		let mut forwarded = Vec::new();
		let mut forward = |timestamp, words: &[u32]| forwarded.push((timestamp, words.to_vec()));
		let seen = Cell::new(0);
		let on_event = |_| {
			let index = seen.replace(seen.get() + 1);

			if index == 0 { CallbackResult::Drop } else { CallbackResult::Passthrough }
		};

		handle_packet(100, &words, 42, &on_event, &mut forward);

		assert_eq!(seen.get(), 2);
		assert_eq!(forwarded, vec![(100, note_off)]);
	}

	#[test]
	fn unparseable_remainder_forwards_unchanged() {
		let words = [vec![0xFFFF_FFFF], midi1_note_on(64)].concat();
		let mut forwarded = Vec::new();
		let mut forward = |timestamp, words: &[u32]| forwarded.push((timestamp, words.to_vec()));
		let seen = Cell::new(0);
		let on_event = |_| {
			seen.set(seen.get() + 1);
			CallbackResult::Drop
		};

		handle_packet(100, &words, 42, &on_event, &mut forward);

		assert_eq!(seen.get(), 0);
		assert_eq!(forwarded, vec![(100, words)]);
	}

	#[test]
	fn observer_only_still_emits_events() {
		let words = midi1_note_on(64);
		let seen = Cell::new(0);
		let on_event = |_| {
			seen.set(seen.get() + 1);
			CallbackResult::Passthrough
		};
		let mut forward = |_, _: &[u32]| {};

		handle_packet(100, &words, 42, &on_event, &mut forward);

		assert_eq!(seen.get(), 1);
	}

	fn event_from_words(source_id: i32, timestamp: u64, words: &[u32]) -> Event {
		let (message, _) = next_message(words).unwrap();
		Event {
			source_id,
			timestamp,
			message: message.rebuffer_into(),
		}
	}

	fn parsed_messages(mut words: &[u32]) -> Vec<Message> {
		let mut messages = Vec::new();

		while let Some((message, consumed)) = next_message(words) {
			messages.push(message.rebuffer_into());
			words = &words[consumed..];
		}

		messages
	}

	fn midi1_note_on(velocity: u8) -> Vec<u32> {
		let mut message = midi2::channel_voice1::NoteOn::<[u32; 4]>::new();
		message.set_channel(u4::new(0));
		message.set_note_number(u7::new(60));
		message.set_velocity(u7::new(velocity));
		message.data().to_vec()
	}

	fn midi1_note_off() -> Vec<u32> {
		let mut message = midi2::channel_voice1::NoteOff::<[u32; 4]>::new();
		message.set_channel(u4::new(0));
		message.set_note_number(u7::new(60));
		message.set_velocity(u7::new(0));
		message.data().to_vec()
	}

	fn midi2_note_on() -> Vec<u32> {
		let mut message = midi2::channel_voice2::NoteOn::<[u32; 4]>::new();
		message.set_channel(u4::new(0));
		message.set_note_number(u7::new(60));
		message.set_velocity(u16::MAX);
		message.data().to_vec()
	}
}
