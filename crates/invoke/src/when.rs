// `when` = generic reactivity, and it is CORE.
//
// A pack declares boolean `var`s and gates behavior on a `WhenExpr` over them.
// That is useful to *any* consumer (a CLI subscribing to "did this toggle?",
// an agent, the app) — it has nothing to do with keyboards. So it lives in
// libinvoke, NOT in the chord engine.
//
// The seam: `common::bool_expr::BoolExpr` is the generic predicate AST (knows
// nothing of vars). The `var` table is this engine's runtime store. The chord
// runtime (system crate) only *reads* this via the resolver `|v| get_var(v)` —
// it never owns vars. Keep it that way: nothing chord/EventTap/MIDI here.

use common::bool_expr::BoolExpr;

pub type VarHandle = u32;

/// A boolean activation gate over pack vars — this engine's instantiation of
/// the generic [`BoolExpr`]. Serializes as the same untagged shape.
pub type WhenExpr = BoolExpr<VarHandle>;

pub mod var {
	use super::VarHandle;
	use common::handle_map::ConcurrentHandleMapU32;
	use std::sync::LazyLock;

	static VARS: LazyLock<ConcurrentHandleMapU32<bool>> = LazyLock::new(ConcurrentHandleMapU32::new);

	pub fn declare_var() -> VarHandle {
		VARS.insert(false)
	}
	pub fn undeclare_var(var: VarHandle) {
		VARS.pin().remove(&var);
	}
	pub fn set_var(var: VarHandle, value: bool) {
		VARS.pin().insert(var, value);
	}
	pub fn get_var(var: VarHandle) -> bool {
		VARS.pin().get(&var).copied().unwrap_or(false)
	}
}
