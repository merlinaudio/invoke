use std::{
	collections::{BTreeSet, HashSet},
	fmt::Display,
};
use thiserror::Error;

use crate::accessibility::{
	attribute::Attribute,
	element::{self, Element},
	value,
};

/// Filter path from root to leaf.
pub type FilterStep = HashSet<Filter>;
pub type FilterPath = Vec<FilterStep>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Conversion error: {0}")]
	Conversion(#[from] value::Error),

	#[error("Element error: {0}")]
	Element(#[from] element::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Filter {
	// ----- Attribute filters just checked for equality -----
	/// Matches if the element's ElementBusy attribute is equal to the given boolean
	ElementBusy(bool),
	/// Matches if the element's Enabled attribute is equal to the given boolean
	Enabled(bool),
	/// Matches if the element's Focused attribute is equal to the given boolean
	Focused(bool),
	/// Matches if the element's Frontmost attribute is equal to the given boolean
	Frontmost(bool),
	/// Matches if the element's Main attribute is equal to the given boolean
	Main(bool),
	/// Matches if the element's Minimized attribute is equal to the given boolean
	Minimized(bool),
	/// Matches if the element's Description attribute is equal to the given string
	Description(MatchString),
	/// Matches if the element's Help attribute is equal to the given string
	Help(MatchString),
	/// Matches if the element's Identifier attribute is equal to the given string
	Identifier(MatchString),
	/// Matches if the element's LabelValue attribute is equal to the given string
	LabelValue(MatchString),
	/// Matches if the element's PlaceholderValue attribute is equal to the given string
	PlaceholderValue(MatchString),
	/// Matches if the element's Role attribute is equal to the given string
	Role(MatchString),
	/// Matches if the element's RoleDescription attribute is equal to the given string
	RoleDescription(MatchString),
	/// Matches if the element's Subrole attribute is equal to the given string
	Subrole(MatchString),
	/// Matches if the element's Title attribute is equal to the given string
	Title(MatchString),
	/// Matches if the element's ValueDescription attribute is equal to the given string
	ValueDescription(MatchString),
	/// Matches if the element's Value attribute is equal to the given string
	Value(MatchString),

	// ----- Other filters -----
	// TODO Has is an expensive operation because this iterates over all children, and then the next step in the Filter path also does.
	// In that case - when there is a next filter path step - why not just store the children we already fetched in this step?
	// Potentially hard to pull off and maybe not worth it - how expensive is it to fetch children? It's just one macOS API call, after all.
	/// Matches if any of the element's direct children match the FilterTree (max. one level deep).
	/// To match a grandchild, nest: `{"has": {"role": "group", "has": {"identifier": "X"}}}`. Expensive operation.
	Has(BTreeSet<Filter>),

	/// Always matches. Useful for grabbing the first element that's being checked.
	Any,
}

impl Filter {
	pub fn matches(&self, element: &mut Element) -> Result<bool, Error> {
		use Filter::*;

		let result = match self {
			ElementBusy(b) => element.attribute(Attribute::ElementBusy)? == *b,
			Enabled(b) => element.attribute(Attribute::Enabled)? == *b,
			Focused(b) => element.attribute(Attribute::Focused)? == *b,
			Frontmost(b) => element.attribute(Attribute::Frontmost)? == *b,
			Main(b) => element.attribute(Attribute::Main)? == *b,
			Minimized(b) => element.attribute(Attribute::Minimized)? == *b,

			Description(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Description)?)?),
			Help(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Help)?)?),
			Identifier(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Identifier)?)?),
			LabelValue(s) => s.matches(<&str>::try_from(element.attribute(Attribute::LabelValue)?)?),
			PlaceholderValue(s) => s.matches(<&str>::try_from(element.attribute(Attribute::PlaceholderValue)?)?),
			Role(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Role)?)?),
			RoleDescription(s) => s.matches(<&str>::try_from(element.attribute(Attribute::RoleDescription)?)?),
			Subrole(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Subrole)?)?),
			Title(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Title)?)?),
			ValueDescription(s) => s.matches(<&str>::try_from(element.attribute(Attribute::ValueDescription)?)?),
			Value(s) => s.matches(<&str>::try_from(element.attribute(Attribute::Value)?)?),

			Has(filters) => {
				// `has` is a child step. It matches when one direct child satisfies every
				// filter in the set, the same way a walk step ANDs its filters against a
				// candidate.
				//
				// A child that lacks an attribute just fails that one filter.
				// It does not abort the search, so a later child can still match.
				// Same tolerance walk applies to its candidates.
				element
					.children()?
					.iter()
					.any(|child| filters.iter().all(|filter| filter.matches(&mut Element::new(child)).unwrap_or(false)))
			}

			Any => true,
		};

		Ok(result)
	}
}

trait Match {
	type T: ?Sized;
	fn matches(&self, compare: impl AsRef<Self::T>) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum MatchString {
	/// Wildcard pattern matching. `?` matches any single character, `*` matches any sequence.
	Glob(String),
	/// Exact identity match. The stored value is already normalized (e.g., AX-prefixed).
	Literal(String),
}

impl Match for MatchString {
	type T = str;

	fn matches(&self, compare: impl AsRef<Self::T>) -> bool {
		match self {
			MatchString::Glob(pattern) => wildmatch::WildMatch::new(pattern).matches(compare.as_ref()),
			MatchString::Literal(value) => value == compare.as_ref(),
		}
	}
}

// ---------------------------------------------------------------------------------------------------------------------

impl Display for MatchString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MatchString::Glob(pattern) => write!(f, "{pattern}"),
			MatchString::Literal(value) => write!(f, "{value}"),
		}
	}
}

impl Display for Filter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Filter::ElementBusy(true) => write!(f, "ElementBusy"),
			Filter::ElementBusy(false) => write!(f, "!ElementBusy"),
			Filter::Enabled(true) => write!(f, "Enabled"),
			Filter::Enabled(false) => write!(f, "!Enabled"),
			Filter::Focused(true) => write!(f, "Focused"),
			Filter::Focused(false) => write!(f, "!Focused"),
			Filter::Frontmost(true) => write!(f, "Frontmost"),
			Filter::Frontmost(false) => write!(f, "!Frontmost"),
			Filter::Main(true) => write!(f, "Main"),
			Filter::Main(false) => write!(f, "!Main"),
			Filter::Minimized(true) => write!(f, "Minimized"),
			Filter::Minimized(false) => write!(f, "!Minimized"),
			Filter::Description(s) => write!(f, "Description({s})"),
			Filter::Help(s) => write!(f, "Help={s}"),
			Filter::Identifier(s) => write!(f, "Identifier={s}"),
			Filter::LabelValue(s) => write!(f, "LabelValue={s}"),
			Filter::PlaceholderValue(s) => write!(f, "PlaceholderValue={s}"),
			Filter::Role(s) => write!(f, "Role={s}"),
			Filter::RoleDescription(s) => write!(f, "RoleDescription={s}"),
			Filter::Subrole(s) => write!(f, "Subrole={s}"),
			Filter::Title(s) => write!(f, "Title={s}"),
			Filter::ValueDescription(s) => write!(f, "ValueDescription={s}"),
			Filter::Value(s) => write!(f, "Value={s}"),
			Filter::Has(filters) => write!(f, "Has({})", filters.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(" || ")),
			Filter::Any => write!(f, "Any"),
		}
	}
}

mod tests {
	#![allow(unused)]
	use super::*;
	use test::{Bencher, black_box};

	#[bench]
	fn bench_glob_dependency(b: &mut Bencher) {
		b.iter(|| black_box(wildmatch::WildMatch::new(black_box("f*rq??d")).matches(black_box("farquaad"))));
	}

	#[bench]
	fn bench_matchstring_glob(b: &mut Bencher) {
		let matchstring = MatchString::Glob("f*rq??d".to_string());
		b.iter(|| black_box(black_box(&matchstring).matches(black_box("faaaaaaaaaaaaaaa☺aaaaaaarqua👨🏻‍❤️‍💋‍👨🏻ad"))));
	}
}
