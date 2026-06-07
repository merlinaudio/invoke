#![feature(test)]

extern crate test;

pub mod chord;
pub mod predicate;

use chord::{Chord, InputResult};
use predicate::Matches;

use common::bool_expr::BoolExpr;

// Shortcut only tracks matching state: `listening`, `when`, chords, and each
// chord's current step.
//
// It returns InputResult plus whether a chord reached the last sequence step.
// The caller decides what that means outside this crate.

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Shortcut<Trigger, Data = ()> {
	pub chords: Vec<Chord<Trigger, Data>>,
	pub when: BoolExpr<u32>,
	#[cfg_attr(feature = "serde", serde(skip_deserializing))]
	// Runtime state, not saved. New and deserialized shortcuts start asleep until `when` is checked.
	pub listening: bool,
}

pub type IsAtEnd = bool;

impl<Trigger, Data> Shortcut<Trigger, Data> {
	pub fn new(chords: Vec<Chord<Trigger, Data>>, when: BoolExpr<u32>) -> Self {
		Self {
			chords,
			when,
			listening: bool::default(),
		}
	}

	/// Re-checks this shortcut's `when` and updates whether it should accept input.
	///
	/// Returns whether any of the shortcut's chords were reset as a result of calling this function.
	pub fn reconcile(&mut self, resolve: &impl Fn(&u32) -> bool) -> bool {
		if self.when.evaluate(resolve) {
			self.listen();
			false
		} else {
			self.sleep()
		}
	}

	pub fn listen(&mut self) {
		self.listening = true;
	}

	/// Stops listening and resets any in-progress chord.
	///
	/// Returns whether any of the shortcut's chords were reset as a result of calling this function.
	pub fn sleep(&mut self) -> bool {
		self.listening = false;
		self.reset()
	}

	/// Resets all chords.
	///
	/// Returns whether any chord was past step 0 before the reset.
	pub fn reset(&mut self) -> bool {
		let mut did_reset = false;
		for chord in &mut self.chords {
			did_reset |= chord.current_step() > 0;
			chord.reset();
		}
		did_reset
	}
}

impl<Trigger: Matches, Data> Shortcut<Trigger, Data> {
	/// Feeds one trigger into this shortcut's chords.
	///
	/// `Triggered` returns immediately. `Advanced` is remembered while other
	/// chords still get a chance. `Dud` only wins if no chord advanced or
	/// triggered.
	///
	/// The boolean is true when a chord reached the last sequence step.
	pub fn input(&mut self, trigger: &Trigger) -> (InputResult, IsAtEnd, Option<&Data>) {
		use InputResult::*;

		if !self.listening {
			return (Dud, false, None);
		}

		let mut best_result = Dud;

		for chord in &mut self.chords {
			let (result, is_at_end, data) = chord.input(trigger);
			if is_at_end {
				// This chord reached the last sequence step.
				// Return immediately with `is_at_end = true`.
				//
				// This means overlapping chords inside the same shortcut stop here.
				// For example:
				//
				//     [Guard(Cmd down), Each(MouseScrollX, Cmd Up)]
				//     [Guard(Cmd down), Guard(Cmd up), Once(Num0)]
				//
				// If the first chord reaches Cmd Up, this function returns before
				// the second chord can keep advancing.
				return (result, true, data);
			}

			match result {
				Triggered => return (Triggered, false, data),
				Advanced => best_result = Advanced,
				Dud => continue, // Still try other chords
			}
		}

		(best_result, false, None)
	}
}

/// A minimal trigger type for exercising the generic engine in tests/benches.
///
/// The crate itself is trigger-agnostic; real trigger vocabularies (keyboard,
/// mouse, MIDI, ...) live in the orchestrator that instantiates `Shortcut<T, _>`.
#[cfg(test)]
pub(crate) mod test_trigger {
	use crate::predicate::{Matches, Number};

	#[derive(Debug, Clone, PartialEq)]
	pub enum TestTrigger {
		KeyDown(u8),
		KeyUp(u8),
		Button(u8),
		Scroll(Number),
	}

	impl Matches for TestTrigger {
		fn matches(&self, other: &Self) -> bool {
			match (self, other) {
				(Self::Scroll(this), Self::Scroll(other)) => this.matches(other),
				_ => self == other,
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{chord::Stmt, predicate::Number, test_trigger::TestTrigger};
	use test::{Bencher, black_box};

	const SHORTCUTS: usize = 10_000;
	const MATCHING_SHORTCUT: usize = 4096;

	#[bench]
	fn bench_10k_shortcuts_one_match_resets_all(bencher: &mut Bencher) {
		let mut shortcuts = shortcuts();
		let keydown_a = TestTrigger::KeyDown(0);
		let matching_scroll = TestTrigger::Scroll(Number::Equals(MATCHING_SHORTCUT as f32));

		bencher.iter(|| {
			black_box(input_shortcuts(black_box(&mut shortcuts), black_box(&keydown_a)));
			black_box(input_shortcuts(black_box(&mut shortcuts), black_box(&matching_scroll)));
		});
	}

	fn shortcuts() -> Vec<Shortcut<TestTrigger, usize>> {
		(0..SHORTCUTS)
			.map(|index| {
				let key = (index % 4) as u8;

				let mut shortcut = Shortcut::new(
					vec![Chord::new(vec![
						Stmt::Guard(TestTrigger::KeyDown(key)),
						Stmt::Each(TestTrigger::Scroll(Number::Equals(index as f32)), Some(TestTrigger::KeyUp(key)), index),
					])],
					BoolExpr::Literal(true),
				);
				shortcut.listen();
				shortcut
			})
			.collect()
	}

	fn input_shortcuts(shortcuts: &mut [Shortcut<TestTrigger, usize>], trigger: &TestTrigger) -> bool {
		use InputResult::*;

		let mut any_triggered = false;

		for shortcut in shortcuts.iter_mut() {
			let (result, is_at_end, data) = shortcut.input(trigger);
			let data = data.copied();

			black_box((&result, is_at_end, data));

			if result == Triggered {
				any_triggered = true;
			}

			if result == Dud || is_at_end {
				shortcut.reset();
			}
		}

		if any_triggered {
			for shortcut in shortcuts.iter_mut() {
				shortcut.reset();
			}
		}

		any_triggered
	}
}

#[cfg(feature = "napi")]
impl<T: serde::Serialize, Data: serde::Serialize> napi::bindgen_prelude::ToNapiValue for Shortcut<T, Data> {
	unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> napi::Result<napi::sys::napi_value> {
		unsafe {
			let json = serde_json::to_value(&val).map_err(|e| napi::Error::from_reason(e.to_string()))?;
			napi::bindgen_prelude::ToNapiValue::to_napi_value(env, json)
		}
	}
}
