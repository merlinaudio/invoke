//! Handle-backed resource guards.
//!
//! Each pack-owned resource — a retained accessibility element, a `when` var, a
//! notification observation — is a handle into some map/set/table.
//!
//! The structs here own such a handle and free it on `Drop`, so when e.g. a [Pack] drops,
//! everything it held is released.

use common::accessibility::ElementHandle;
use common::main_thread::MainThread;
use observer::accessibility::NotificationRegistrationHandle;

use crate::when::{VarHandle, var::undeclare_var};
use crate::{ax, dispose_element};

// Each guard is a plain struct with its own `Drop` — no generic wrapper, since
// `Drop` already does the job.
//
// A guard is not its handle. The handle is `Copy` and goes on the wire; the
// guard owns the resource and frees it on drop. A type can't be both `Copy`
// and `Drop`.

/// Owns a retained accessibility element; disposes it on drop.
pub struct Element(pub ElementHandle);
impl Drop for Element {
	fn drop(&mut self) {
		dispose_element(self.0.into());
	}
}

/// Owns a `when` reactivity var; undeclares it on drop.
pub struct Var(pub VarHandle);
impl Drop for Var {
	fn drop(&mut self) {
		undeclare_var(self.0);
	}
}

/// Owns an accessibility-notification observation; unobserves it on drop.
pub struct Notification(pub NotificationRegistrationHandle);
impl Drop for Notification {
	fn drop(&mut self) {
		let handle = self.0;
		MainThread::dispatch(move || {
			_ = ax::unobserve_element_notification(handle).inspect_err(|e| log::error!("Couldn't un-observe element notification: {e:?}"));
		});
	}
}
