//! Small, app-agnostic observers for platform event sources.
//!
//! An observer turns a noisy upstream surface into a Rust event API that an app
//! can actually use. That API is allowed to be opinionated: callers should not
//! need to know which CoreGraphics field, AX notification object, Workspace
//! notification, or CoreMIDI packet shape produced an event.
//!
//! The boundary is still generic. This crate does not know about Invoke,
//! shortcuts, commands, or any app-specific trigger language. Apps map observer
//! events into their own concepts.

#![feature(fn_traits)]
#![feature(variant_count)]

use std::fmt::Debug;

#[cfg(feature = "eventtap")]
pub mod eventtap;

#[cfg(feature = "accessibility")]
pub mod accessibility;

#[cfg(feature = "workspace")]
pub mod workspace;

#[cfg(feature = "midi")]
pub mod midi;

/// A platform event source with an explicit active/asleep lifecycle.
pub trait Observer {
	type Error: std::error::Error;
	type Event: Clone + PartialEq + Debug;

	/// Make sure this observer is listening.
	///
	/// Calling this twice should be boring. If MIDI is already connected to
	/// source A, don't connect to source A again. If eventtap is already in the
	/// run loop, don't add another run loop source.
	fn listen(&self) -> Result<(), Self::Error>;

	/// Stop delivering events.
	fn sleep(&self) -> Result<(), Self::Error>;
}

/// Automatically calls `sleep()` when dropped.
///
/// This is just a guard, not a deduper.
///
/// If the app says "make sure this is listening" three times, this will call
/// inner.listen() three times. So inner.listen() needs to be boring when it is
/// already listening.
///
/// All `MyObserver::new()` functions should return this type,
/// and implement extra methods (like `MyObserver::register_pid()`) on `ObserverGuard<MyObserver>`.
#[derive(Debug)]
pub struct ObserverGuard<O: Observer> {
	inner: O,
}

impl<O: Observer> ObserverGuard<O> {
	pub fn toggle(&self, active: bool) -> Result<(), O::Error> {
		if active { self.listen() } else { self.sleep() }
	}
}

impl<O: Observer> Observer for ObserverGuard<O> {
	type Error = O::Error;
	type Event = O::Event;

	// This means "make sure inner is listening", not "add another listener".
	fn listen(&self) -> Result<(), Self::Error> {
		self.inner.listen()
	}
	fn sleep(&self) -> Result<(), Self::Error> {
		self.inner.sleep()
	}
}

impl<O: Observer> Drop for ObserverGuard<O> {
	fn drop(&mut self) {
		if let Err(e) = self.sleep() {
			log::warn!("Failed to sleep() observer in ObserverGuard.drop(): {e:?}");
		}
	}
}
