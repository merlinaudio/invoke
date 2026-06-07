use crate::error::*;

/// Resolve an optional bundle ID to a PID.
pub fn resolve_location(app: Option<&str>) -> Result<Option<u32>> {
	match app {
		Some(bid) => Ok(Some(common::process::pid_for_bundle(bid).err_code("NoRunningApp")?)),
		None => Ok(None),
	}
}
