use std::fmt::{Debug, Display};
use thiserror::Error;

use crate::predicate::Matches;

#[derive(Debug, Error)]
pub enum Error {}

/// A chord is a mini program / state machine type thingy that represents a sequence of steps that must be followed to trigger a function.
///
/// For example, it can be used to represent a function that is triggered every time the user holds down a key and then scrolls with their mouse.
///
/// This a mouthful, so let me show an example instead:
///
/// ```rs
/// let chord = Chord::new(vec![
///     Guard(KeyboardKeysDown(Cmd)), // Require holding Cmd, then,
///     Each(MouseScrollX, usize::MAX), // every time the user scrolls,
/// ], action_fn); // run this callback
/// ```
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Chord<Trigger, Data = ()> {
	/// The steps that make up the chord.
	///
	/// For example:
	///
	///```rs
	/// use Trigger::*;
	///
	/// // A huge sequence just to show all the possibilities. Most sequences are just a couple steps long, you'd never
	/// // have a long incantation like this in practice, because shortcuts are supposed to be... short.
	///
	/// sequence = vec![
	///   // This is a simple key press - the user presses a key and releases it:
	///
	///   Guard(KeyboardKeysDown(C)),
	///   Guard(KeyboardKeysUp(C)),
	///
	///   // This is a simple key press with modifiers.
	///   // The user presses some keys (cmd, option, ...) in any order,
	///   // and then one key,
	///   // and then releases all of them in any order:
	///
	///   Guard(KeyboardKeysDown(Control, Shift))   // Press Control and Shift, or Shift and Control
	///   Guard(KeyboardKeysDown(A)),               // Press A
	///   Guard(KeyboardKeysUp(Control, Shift, A)), // Release Control, Shift, and A in any order
	///
	///   Guard(KeyboardKeysDown(A, Cmd)),          // User must press and hold A and Cmd in any order.
	///                                             // This is uncommon, because it would allow pressing A before Cmd, and still work.
	///
	///   Guard(KeyboardKeysUp(A)),                 // Then must release A (Cmd remains pressed).
	///                                             // If Cmd is released at this point, the Chord doesn't match:
	///                                             // our program will check for any shortcut that has KeyboardKeysUp(Cmd)
	///                                             // as the next step.
	///                                             // This isn't the case here, so the sequence is aborted.
	///
	///   Each(MouseScrollX, KeyboardKeysUp(Cmd))     // Each time the user scrolls, the shortcut is triggered, until the Cmd key is released.
	/// ];
	/// ```
	pub sequence: Sequence<Trigger, Data>,

	#[cfg_attr(feature = "serde", serde(skip))]
	current_step: usize,
}

impl<Trigger, Data> Chord<Trigger, Data> {
	pub fn new(sequence: Sequence<Trigger, Data>) -> Self {
		Self { sequence, current_step: 0 }
	}

	pub fn current_expr(&self) -> Option<&Stmt<Trigger, Data>> {
		self.sequence.get(self.current_step)
	}

	pub fn current_step(&self) -> usize {
		self.current_step
	}

	pub fn is_at_end(&self) -> bool {
		self.current_step == self.sequence.len()
	}

	/// Manually advances the chord. Use this to skip the current step.
	///
	/// If the chord was at the end of the sequence before calling this, it will be reset to start from the beginning.
	///
	/// You probably want to use `input()` instead, which advances the chord according to the current step's requirements.
	pub fn advance(&mut self) -> InputResult {
		self.current_step += 1;
		InputResult::Advanced // Just for convencience in `input()`.
	}

	pub fn reset(&mut self) {
		self.current_step = 0;
	}
}

impl<Trigger: Matches, Data> Chord<Trigger, Data> {
	/// Feed some input to the chord to move to the next step in its sequence.
	///
	/// Returns a bool indicating whether the action should be run (i.e. shortcut should be called).
	///
	/// ---
	///
	/// ### Explanation of return type:
	///
	/// Returns `None` when the sequence is finished, either by reaching the end or by an input that doesn't match the current step.\
	/// In this case, you likely want to call `.reset()` to start over. If in this state, `input()` will keep returning `None` until the Chord is reset.
	///
	/// Returns `Some(bool)`, where `true` means the action should be run, and `false` means the action shouldn't be run (do nothing).
	pub fn input(&mut self, trigger: &Trigger) -> (InputResult, bool, Option<&Data>) {
		// THIS FUNCTION IS CALLED VERY OFTEN. ASSUME EACH MOUSE PIXEL MOVEMENT, EACH KEYPRESS, EACH SCROLL, etc.
		use InputResult::*;
		use Stmt::*;

		let current_step = self.current_step;
		let Some(current_expr) = self.sequence.get(current_step) else {
			return (Dud, false, None);
		};

		match current_expr {
			Guard(wanted) if trigger.matches(wanted) => {
				self.current_step += 1;
				(Advanced, self.is_at_end(), None)
			}

			Await(wanted) if trigger.matches(wanted) => {
				self.current_step += 1;
				(Advanced, self.is_at_end(), None)
			}
			Await(_) => (Advanced, false, None), // Permissive - stays alive on non-match

			Once(wanted, data) if trigger.matches(wanted) => {
				self.current_step += 1;
				(Triggered, self.is_at_end(), Some(data))
			}

			Each(_, Some(until), _) if trigger.matches(until) => {
				self.current_step += 1;
				(Advanced, self.is_at_end(), None)
			} // Advance when `until` is input.
			Each(wanted, _, data) if trigger.matches(wanted) => (Triggered, false, Some(data)), // Call the action every time `wanted`` is input, and `until` doesn't match. (Otherwise case above would have taken precedence)
			Each(_, _, _) => (Advanced, false, None), // While in an "Each" loop, signal that the chord is still active and participating. Signaling `Dud` means the chord is over/failed.

			_ => (Dud, false, None),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "napi", napi_derive::napi)]
pub enum InputResult {
	// Consumers that gate downstream delivery usually treat this result on a
	// given input as "drop this input from downstream."
	//
	// As of writing, only Stmt's `Once()` and `Each()` trigger step return
	// `Triggered`; `Guard`/`Await`/`Each`'s `until` return `Advanced` and do
	// **not** drop. So e.g. Cmd keeps reaching the focused app while also
	// satisfying a chord's prefix.
	//
	// All of this is subject to change and may be outdated at the time of reading.
	//
	/// Chord matched input and signaled that it was triggered/activated.
	Triggered,

	/// Chord matched input, but didn't signal that it was triggered/activated.
	Advanced,

	/// Chord did not match input. In most implementations, the caller probably wants to reset the chord/shortcut if this happens.
	Dud,
}

#[derive(Debug, Error)]
pub enum InputError {
	#[error("Stmt invalid or not implemented")]
	InvalidStmt,
}

impl<Trigger: Debug, Data: Debug> Display for Chord<Trigger, Data> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Chord({} / {:?})", self.current_step, self.sequence)
	}
}

pub type Sequence<Trigger, Data = ()> = Vec<Stmt<Trigger, Data>>;

/// Represents how a `Chord` should behave when it receives input.
///
/// > How should the Chord react to this input?
/// > Should it signal 'triggered!',
/// > should it advance but signal to 'do nothing',
/// > should it get stuck in a loop and signal 'triggered!' every time until some condition is met?
///
/// - Guard(Trigger) means the trigger must be activated, but doesn't call the shortcut.
/// - Once(Trigger) means the trigger must be activated, and calls the shortcut.
/// - Each(Trigger) means each time the trigger is activated, the shortcut is called.
///
/// etc.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum Stmt<Trigger, Data = ()> {
	//////////////////////
	/// Non-triggering ///
	//////////////////////

	/// Trigger must be activated once before moving on to the next step in the sequence. Does not call shortcut.
	Guard(Trigger),

	/// Like Guard, but permissive - ignores non-matching inputs instead of failing.
	///
	/// **Why this exists**: Hold shortcuts like `once(down(alt)) → guard(up(alt))` would get
	/// canceled whenever the user clicks the mouse or presses any other key while holding.
	/// This happens because Guard returns Dud for non-matching inputs, triggering a reset.
	///
	/// Await solves this by returning Advanced (stay alive) instead of Dud (abort) for
	/// non-matching inputs. Use it for trailing deactivations where the user should be
	/// free to perform other actions while waiting for the release.
	///
	/// - Guard = "this must happen next, any other input aborts"
	/// - Await = "wait for this trigger, ignore other inputs"
	Await(Trigger),

	// Commented out because I don't see a use case for this yet and I don't like the name "Repeat" as it could be mistaken to mean the same thing as `Each`.
	/// Trigger must be activated multiple times (second field) before moving on to the next step. Does not call shortcut.
	// Repeat(Trigger, usize),

	//////////////////
	/// Triggering ///
	//////////////////

	/// Shortcut is called once the trigger is activated.
	///
	/// Great for building hold-shortcuts:
	/// ```rs
	/// sequence = vec![
	///   Once(KeyboardKeysDown(C)), // Triggers now,
	///   Once(KeyboardKeysUp(C)),   // and again when the key is released.
	///                              //
	///                              // Then the caller can figure out how to handle this based on the trigger;
	///                              //
	///                              // For example, it could require [what it considers as "Hold Shortcuts"] to have sequences
	///                              // that must end in this (Once-Down, Once-Up) pattern.
	///                              //
	///                              // Either way, it's not up to Chord to interpret results - it just advances its sequence based on these rules.
	/// ];
	/// ```
	Once(Trigger, Data),

	/// Shortcut is called every time the trigger is activated.
	///
	/// Will not advance until the trigger in the 2nd field has been activated.
	///
	/// Great for building mousescroll and mousedrag shortcuts:
	///
	/// ```rs
	/// scroll_sequence = vec![
	///   Guard(KeyboardKeysDown(Cmd)),             // Require holding Cmd
	///   Each(MouseScrollX, KeyboardKeysUp(Cmd)),  // Each time the user scrolls, the shortcut is triggered, until KeyboardKeysUp(Cmd)
	/// ];
	///
	/// drag_sequence = vec![
	///   Guard(KeyboardKeysDown(Cmd)),                      // Require holding Cmd
	///   Each(MouseButtonDown(Left), MouseButtonUp(Left)),  // When the user holds down left mouse, the shortcut is triggered
	/// ];
	/// ```
	Each(Trigger, Option<Trigger>, Data),
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::predicate::Number;
	use crate::test_trigger::TestTrigger;
	use test::{Bencher, black_box};

	// Await must return Advanced on non-match (not Dud like Guard).
	// If this breaks, hold-shortcuts abort when user clicks mouse while holding a key.
	#[test]
	fn await_stays_alive_on_nonmatch() {
		let mut chord: Chord<TestTrigger> = Chord::new(vec![Stmt::Await(TestTrigger::KeyUp(0))]);

		// Feed a completely different trigger multiple times
		let unrelated = TestTrigger::Button(0);
		assert_eq!(chord.input(&unrelated), (InputResult::Advanced, false, None));
	}

	// Each must check `until` before `wanted`. If reordered, loop fires once extra on exit.
	#[test]
	fn each_until_basic() {
		let mut chord = Chord::new(vec![Stmt::Each(TestTrigger::Scroll(Number::Positive), Some(TestTrigger::KeyUp(0)), ())]);

		// Feed the `wanted` trigger - should trigger multiple times
		let wanted_trigger = TestTrigger::Scroll(Number::Positive);
		assert_eq!(chord.input(&wanted_trigger), (InputResult::Triggered, false, Some(&())));
		assert_eq!(chord.input(&wanted_trigger), (InputResult::Triggered, false, Some(&())));
		assert_eq!(chord.input(&wanted_trigger), (InputResult::Triggered, false, Some(&())));

		// Feed the `until` trigger - should advance out of loop, NOT trigger
		let until_trigger = TestTrigger::KeyUp(0);
		assert_eq!(chord.input(&until_trigger), (InputResult::Advanced, true, None));

		// Subsequent `wanted` trigger should not trigger again
		assert_ne!(chord.input(&wanted_trigger).0, InputResult::Triggered);
	}

	#[test]
	fn once_returns_activation_data() {
		let mut chord = Chord::new(vec![Stmt::Once(TestTrigger::KeyDown(0), "payload")]);

		assert_eq!(chord.input(&TestTrigger::KeyDown(0)), (InputResult::Triggered, true, Some(&"payload")));
	}

	#[bench]
	pub fn bench_keydown_scroll_keyup(b: &mut Bencher) {
		let mut chord = Chord::new(vec![
			Stmt::Guard(TestTrigger::KeyDown(0)),
			Stmt::Each(TestTrigger::Scroll(1.0.into()), Some(TestTrigger::KeyUp(0)), ()),
		]);

		let keydown_a = &TestTrigger::KeyDown(0);
		let keyup_a = &TestTrigger::KeyUp(0);
		let mouse_scroll = &TestTrigger::Scroll(1.0.into());

		b.iter(move || {
			black_box(chord.input(black_box(keydown_a)));

			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));
			black_box(chord.input(black_box(mouse_scroll)));

			black_box(chord.input(black_box(keyup_a)));
			chord.reset();
		});
	}
}
