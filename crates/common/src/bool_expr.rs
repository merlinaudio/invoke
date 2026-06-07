//! A generic boolean-expression AST. Nothing here knows what a variable *is* —
//! it is `Literal`, `And`, `Or`, `Not`, and an opaque `Var(V)` leaf the caller
//! resolves. Reusable in any project that needs a serializable predicate tree.
//!
//! ```rs
//! // !get(1) && (get(73) || get(592))
//! And { and: vec![
//!   Not { not: Box::new(Var { var: 1 }) },
//!   Or  { or:  vec![Var { var: 73 }, Var { var: 592 }] },
//! ]};
//! ```
//!
//! Serialized form (`#[serde(untagged)]`): a bare bool is a `Literal`; the rest
//! are single-key objects (`{"and":[…]}`, `{"not":…}`, `{"var":…}`).

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum BoolExpr<Var> {
	/// Constant `true`/`false`. Listed first so `#[serde(untagged)]` matches bare booleans here.
	Literal(bool),
	/// `&&` — all sub-expressions are `true`.
	And { and: Vec<BoolExpr<Var>> },
	/// `||` — any sub-expression is `true`.
	Or { or: Vec<BoolExpr<Var>> },
	/// `!` — the sub-expression is `false`.
	Not { not: Box<BoolExpr<Var>> },
	/// An opaque variable; `resolve` decides its truth.
	Var { var: Var },
}

impl<Var> BoolExpr<Var> {
	/// Evaluate the tree, resolving each `Var` leaf via `resolve`.
	pub fn evaluate(&self, resolve: &impl Fn(&Var) -> bool) -> bool {
		match self {
			BoolExpr::Literal(value) => *value,
			BoolExpr::And { and } => and.iter().all(|expr| expr.evaluate(resolve)),
			BoolExpr::Or { or } => or.iter().any(|expr| expr.evaluate(resolve)),
			BoolExpr::Not { not } => !not.evaluate(resolve),
			BoolExpr::Var { var } => resolve(var),
		}
	}

	/// Whether any leaf is `var`.
	pub fn has_var(&self, var: &Var) -> bool
	where
		Var: PartialEq,
	{
		match self {
			BoolExpr::Literal(_) => false,
			BoolExpr::And { and } => and.iter().any(|expr| expr.has_var(var)),
			BoolExpr::Or { or } => or.iter().any(|expr| expr.has_var(var)),
			BoolExpr::Not { not } => not.has_var(var),
			BoolExpr::Var { var: v } => v == var,
		}
	}
}

#[cfg(all(test, feature = "serde"))]
mod tests {
	use super::*;

	#[test]
	fn literal_roundtrips_as_bare_bool() {
		assert_eq!(serde_json::to_string(&BoolExpr::<u32>::Literal(false)).unwrap(), "false");
		assert_eq!(serde_json::to_string(&BoolExpr::<u32>::Literal(true)).unwrap(), "true");
		assert_eq!(serde_json::from_str::<BoolExpr<u32>>("false").unwrap(), BoolExpr::Literal(false));
		assert_eq!(serde_json::from_str::<BoolExpr<u32>>("true").unwrap(), BoolExpr::Literal(true));
	}

	#[test]
	fn object_variants_roundtrip_with_field_names() {
		let expr: BoolExpr<u32> = BoolExpr::And {
			and: vec![
				BoolExpr::Not {
					not: Box::new(BoolExpr::Var { var: 1 }),
				},
				BoolExpr::Or {
					or: vec![BoolExpr::Literal(true), BoolExpr::Var { var: 2 }],
				},
			],
		};
		let json = serde_json::to_string(&expr).unwrap();
		assert_eq!(json, r#"{"and":[{"not":{"var":1}},{"or":[true,{"var":2}]}]}"#);
		assert_eq!(serde_json::from_str::<BoolExpr<u32>>(&json).unwrap(), expr);
	}

	#[test]
	fn evaluate_resolves_vars() {
		let expr: BoolExpr<u32> = BoolExpr::And {
			and: vec![
				BoolExpr::Not {
					not: Box::new(BoolExpr::Var { var: 1 }),
				},
				BoolExpr::Or {
					or: vec![BoolExpr::Var { var: 2 }, BoolExpr::Literal(false)],
				},
			],
		};
		assert!(expr.evaluate(&|v: &u32| *v == 2)); // !false && (true || false)
		assert!(!expr.evaluate(&|v: &u32| *v == 1)); // !true  && …
	}
}
