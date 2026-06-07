//! Reusable matcher types used by shortcut fields.
//!
//! These are orthogonal to triggers. A trigger answers "what chord-relevant
//! thing happened?", while a matcher answers "how should one field compare?".

/// Symmetric overlap matching.
///
/// Implementations should answer:
///
/// > Could these two values both match the same concrete input?
pub trait Matches {
	fn matches(&self, other: &Self) -> bool;
}

pub trait Concrete<T> {
	fn concrete(&self) -> Option<T>;
}

/// Discrete matcher over an opaque value domain.
///
/// This is for equality and set-membership semantics only.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum Discrete<T> {
	/// Matches any value in the domain.
	Any,
	/// Matches exactly this value.
	Equals(T),
	/// Matches any one of these values.
	In(Vec<T>),
}

impl<T: PartialEq> Matches for Discrete<T> {
	fn matches(&self, other: &Self) -> bool {
		use Discrete::*;

		match (self, other) {
			(Any, _) | (_, Any) => true,
			(Equals(a), Equals(b)) => a == b,
			(Equals(value), In(values)) | (In(values), Equals(value)) => values.iter().any(|candidate| candidate == value),
			(In(left), In(right)) => left.iter().any(|left| right.iter().any(|right| left == right)),
		}
	}
}

impl<T> From<T> for Discrete<T> {
	fn from(value: T) -> Self {
		Discrete::Equals(value)
	}
}

impl<T: Clone> Concrete<T> for Discrete<T> {
	fn concrete(&self) -> Option<T> {
		match self {
			Discrete::Equals(value) => Some(value.clone()),
			_ => None,
		}
	}
}

// Read this before you touch Number or add a new variant.
//
// Trigger plays two roles. On one side, it's how we produce inputs — real HID
// events get wrapped in a Trigger and fed into chords. On the other side, it's
// the pattern language used inside a chord to say what should match. Those two
// roles have opposite requirements:
//
//   - The producer has to emit the *most specific* variant, i.e.
//     Equals(v), carrying the actual number. That's the ground truth.
//   - The matcher (in a chord) can be as broad or narrow as it wants:
//     Positive, Negative, Any, GreaterThan(3.0), Between(1.0, 10.0),
//     Equals(5.0), whatever. It's a predicate over the ground truth.
//
// matches() is symmetric, so it can answer "does this concrete number satisfy
// this predicate?" for any pair. That works fine — as long as the producer side
// is concrete.
//
// If you try to be clever and have the producer emit a broad variant like
// Positive (because "direction is all we care about"), you break every narrower
// predicate. A chord asking for GreaterThan(3.0) can never be satisfied against
// an incoming Positive — there's no magnitude to compare. The richer variants
// become dead code that looks like it works.
//
// So: producer = ground truth (Equals(v)). Matcher = predicate (anything). Any
// new variant you add just needs to be expressible as a predicate over concrete
// values; these From impls don't change.
//
// NaN note: if an f32::NAN ever reaches Equals, every comparison in matches()
// returns false, so the chord silently won't fire. HID scroll deltas are never
// NaN in practice, so we don't filter — but don't construct Equals(NaN) by
// hand.

/// Numeric matcher.
///
/// This stays flat on purpose. We want one self-contained numeric matcher type,
/// not a nested generic matcher model.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum Number {
	/// Matches exactly this number.
	Equals(f32),
	/// Any negative number.
	Negative,
	/// Any number.
	Any,
	/// Any positive number.
	Positive,
	/// Strictly greater than this value.
	GreaterThan(f32),
	/// Strictly less than this value.
	LessThan(f32),
	/// Exclusive range: low < value < high.
	Between(f32, f32),
}

impl From<f32> for Number {
	fn from(value: f32) -> Self {
		Number::Equals(value)
	}
}

impl From<i32> for Number {
	fn from(value: i32) -> Self {
		Number::Equals(value as f32)
	}
}

impl Concrete<f32> for Number {
	fn concrete(&self) -> Option<f32> {
		match self {
			Number::Equals(value) => Some(*value),
			_ => None,
		}
	}
}

impl Matches for Number {
	fn matches(&self, other: &Self) -> bool {
		use Number::*;

		match (self, other) {
			(Any, Equals(v)) | (Equals(v), Any) => !v.is_nan(),
			(Any, _) | (_, Any) => true,

			// Direction matching (e.g. scroll)
			(Negative, Negative) | (Positive, Positive) => true,
			(Negative, Positive) | (Positive, Negative) => false,

			// Exact value matching
			(Equals(a), Equals(b)) => a == b,

			// EqualTo vs direction
			(Equals(v), Positive) | (Positive, Equals(v)) => *v > 0.0,
			(Equals(v), Negative) | (Negative, Equals(v)) => *v < 0.0,

			// EqualTo vs comparison
			(Equals(v), GreaterThan(t)) | (GreaterThan(t), Equals(v)) => v > t,
			(Equals(v), LessThan(t)) | (LessThan(t), Equals(v)) => v < t,
			(Equals(v), Between(lo, hi)) | (Between(lo, hi), Equals(v)) => v > lo && v < hi,

			// Broad vs broad comparison
			(Negative, GreaterThan(v)) | (GreaterThan(v), Negative) => !v.is_nan() && *v < 0.0,
			(Negative, LessThan(v)) | (LessThan(v), Negative) => !v.is_nan() && *v > f32::NEG_INFINITY,
			(Negative, Between(lo, hi)) | (Between(lo, hi), Negative) => !lo.is_nan() && !hi.is_nan() && lo < hi && *lo < 0.0,

			(Positive, GreaterThan(v)) | (GreaterThan(v), Positive) => !v.is_nan() && *v < f32::INFINITY,
			(Positive, LessThan(v)) | (LessThan(v), Positive) => !v.is_nan() && *v > 0.0,
			(Positive, Between(lo, hi)) | (Between(lo, hi), Positive) => !lo.is_nan() && !hi.is_nan() && lo < hi && *hi > 0.0,

			(GreaterThan(a), GreaterThan(b)) => !a.is_nan() && !b.is_nan() && (*a < f32::INFINITY || *b < f32::INFINITY),
			(LessThan(a), LessThan(b)) => !a.is_nan() && !b.is_nan() && (*a > f32::NEG_INFINITY || *b > f32::NEG_INFINITY),
			(GreaterThan(lo), LessThan(hi)) | (LessThan(hi), GreaterThan(lo)) => !lo.is_nan() && !hi.is_nan() && lo < hi,

			(GreaterThan(bound), Between(lo, hi)) | (Between(lo, hi), GreaterThan(bound)) => {
				!bound.is_nan() && !lo.is_nan() && !hi.is_nan() && lo < hi && *hi > *bound
			}

			(LessThan(bound), Between(lo, hi)) | (Between(lo, hi), LessThan(bound)) => {
				!bound.is_nan() && !lo.is_nan() && !hi.is_nan() && lo < hi && *lo < *bound
			}

			(Between(a_lo, a_hi), Between(b_lo, b_hi)) => {
				!a_lo.is_nan() && !a_hi.is_nan() && !b_lo.is_nan() && !b_hi.is_nan() && a_lo < a_hi && b_lo < b_hi && a_lo < b_hi && b_lo < a_hi
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn discrete_matches_by_overlap() {
		assert!(Discrete::<u16>::Any.matches(&Discrete::Equals(5)));
		assert!(Discrete::Equals(5).matches(&Discrete::Equals(5)));
		assert!(!Discrete::Equals(5).matches(&Discrete::Equals(6)));
		assert!(Discrete::Equals(5).matches(&Discrete::In(vec![1, 5, 9])));
		assert!(!Discrete::Equals(5).matches(&Discrete::In(vec![1, 6, 9])));
		assert!(Discrete::In(vec![1, 5]).matches(&Discrete::In(vec![5, 9])));
		assert!(!Discrete::In(vec![1, 2]).matches(&Discrete::In(vec![3, 4])));
	}

	#[test]
	fn number_from_impls_emit_concrete_matchers() {
		assert_eq!(Number::from(0i32), Number::Equals(0.0));
		assert_eq!(Number::from(1i32), Number::Equals(1.0));
		assert_eq!(Number::from(-1i32), Number::Equals(-1.0));
		assert_eq!(Number::from(-0.3f32), Number::Equals(-0.3));
	}

	#[test]
	fn number_matcher_basic() {
		let incoming = Number::Equals(5.0);
		assert!(incoming.matches(&Number::Any));
		assert!(incoming.matches(&Number::Positive));
		assert!(!incoming.matches(&Number::Negative));
		assert!(incoming.matches(&Number::GreaterThan(3.0)));
		assert!(!incoming.matches(&Number::GreaterThan(10.0)));
		assert!(incoming.matches(&Number::LessThan(10.0)));
		assert!(!incoming.matches(&Number::LessThan(3.0)));
		assert!(incoming.matches(&Number::Between(1.0, 10.0)));
		assert!(!incoming.matches(&Number::Between(10.0, 20.0)));
		assert!(incoming.matches(&Number::Equals(5.0)));
		assert!(!incoming.matches(&Number::Equals(6.0)));
	}

	#[test]
	fn ordered_matchers_respect_exclusive_boundaries() {
		assert!(Number::Between(1.0, 4.0).matches(&Number::LessThan(10.0)));
		assert!(Number::Between(1.0, 4.0).matches(&Number::LessThan(4.0)));
		assert!(!Number::Between(1.0, 4.0).matches(&Number::LessThan(1.0)));

		assert!(Number::Between(1.0, 4.0).matches(&Number::GreaterThan(1.0)));
		assert!(!Number::Between(1.0, 4.0).matches(&Number::GreaterThan(4.0)));
		assert!(!Number::Between(1.0, 4.0).matches(&Number::GreaterThan(10.0)));
	}

	#[test]
	fn ordered_matchers_overlap_when_ranges_share_space() {
		assert!(Number::GreaterThan(3.0).matches(&Number::LessThan(10.0)));
		assert!(!Number::GreaterThan(3.0).matches(&Number::LessThan(3.0)));

		assert!(Number::Between(1.0, 4.0).matches(&Number::Between(3.0, 6.0)));
		assert!(!Number::Between(1.0, 4.0).matches(&Number::Between(4.0, 6.0)));
		assert!(!Number::Between(1.0, 4.0).matches(&Number::Between(-2.0, 1.0)));
	}

	#[test]
	fn direction_matchers_overlap_only_when_signs_allow_it() {
		assert!(Number::GreaterThan(3.0).matches(&Number::Positive));
		assert!(Number::Positive.matches(&Number::LessThan(1.0)));
		assert!(!Number::Positive.matches(&Number::LessThan(0.0)));

		assert!(Number::Negative.matches(&Number::GreaterThan(-1.0)));
		assert!(!Number::Negative.matches(&Number::GreaterThan(0.0)));
		assert!(!Number::Negative.matches(&Number::Positive));
	}

	#[test]
	fn ordered_matching_is_symmetric() {
		let left = Number::Between(1.0, 4.0);
		let right = Number::LessThan(4.0);
		assert_eq!(left.matches(&right), right.matches(&left));

		let left = Number::GreaterThan(3.0);
		let right = Number::Positive;
		assert_eq!(left.matches(&right), right.matches(&left));
	}

	#[test]
	fn nan_matches_nothing() {
		assert!(!Number::Equals(f32::NAN).matches(&Number::Any));
		assert!(!Number::Any.matches(&Number::Equals(f32::NAN)));
	}
}
