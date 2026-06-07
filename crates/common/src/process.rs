use std::path::PathBuf;

use objc2_app_kit::NSRunningApplication;
use objc2_foundation::{NSFileManager, NSSearchPathDirectory, NSSearchPathDomainMask, NSString};

/// Resolve a bundle identifier to a running process ID.
pub fn pid_for_bundle(bundle_id: &str) -> Option<u32> {
	let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(bundle_id));
	let app = apps.firstObject()?;
	let pid = app.processIdentifier();
	if pid <= 0 {
		return None;
	}
	Some(pid as u32)
}

/// All currently running applications.
pub fn running_applications() -> objc2::rc::Retained<objc2_foundation::NSArray<NSRunningApplication>> {
	use objc2_app_kit::NSWorkspace;
	NSWorkspace::sharedWorkspace().runningApplications()
}

/// Get the user's Application Support directory.
pub fn application_support_dir() -> Option<PathBuf> {
	let file_manager = NSFileManager::defaultManager();
	let urls = file_manager.URLsForDirectory_inDomains(NSSearchPathDirectory::ApplicationSupportDirectory, NSSearchPathDomainMask::UserDomainMask);
	let url = urls.firstObject()?;
	url.path().map(|p| PathBuf::from(p.to_string()))
}
