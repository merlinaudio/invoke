//! NSWorkspace application lifecycle observer.
//!
//! This module observes the small set of workspace notifications that matter to
//! app lifecycle: activation, deactivation, and termination. It exposes those as
//! [`Event`] values instead of leaking arbitrary [`NSNotification`] names.
//!
//! The observer's job stops at app lifecycle. Consumers decide whether an
//! activation should enable an event tap, refresh app state, or do nothing.

use std::{collections::HashMap, ptr::NonNull};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_app_kit::{
	NSRunningApplication, NSWorkspace, NSWorkspaceDidActivateApplicationNotification, NSWorkspaceDidDeactivateApplicationNotification,
	NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSNotificationName, NSString};
use thiserror::Error;

use crate::{Observer, ObserverGuard};

#[derive(Debug, Error)]
pub enum Error {
	#[error("Failed to create workspace observer")]
	CreationFailed,

	#[error("Unsupported notification type")]
	UnsupportedNotification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
	ApplicationActivated(Retained<NSRunningApplication>),
	ApplicationDeactivated(Retained<NSRunningApplication>),
	ApplicationTerminated(Retained<NSRunningApplication>),
}

type EventHandlerRcBlock = RcBlock<dyn Fn(NonNull<NSNotification>)>;

#[derive(Debug)]
pub struct WorkspaceObserver {
	notification_center: Retained<NSNotificationCenter>,
	added_notifications: HashMap<Notification, EventHandlerRcBlock>,
	on_event_block: RcBlock<dyn Fn(NonNull<NSNotification>)>,
}

/// Notification type for `.add_notification(Notification, Callback)`
///
/// The type passed to the callback, carrying data about what application was activated/deactivated/terminated/... is [`Event`]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Notification {
	ApplicationActivated,
	ApplicationDeactivated,
	ApplicationTerminated,
}

impl TryInto<&'static NSNotificationName> for Notification {
	type Error = ();

	fn try_into(self) -> Result<&'static NSNotificationName, Self::Error> {
		match self {
			Notification::ApplicationActivated => Ok(unsafe { NSWorkspaceDidActivateApplicationNotification }),
			Notification::ApplicationDeactivated => Ok(unsafe { NSWorkspaceDidDeactivateApplicationNotification }),
			Notification::ApplicationTerminated => Ok(unsafe { NSWorkspaceDidTerminateApplicationNotification }),
		}
	}
}
impl TryFrom<&NSString> for Notification {
	type Error = ();

	fn try_from(notification_name: &NSString) -> Result<Self, Self::Error> {
		unsafe {
			Ok(match notification_name {
				n if n == NSWorkspaceDidActivateApplicationNotification => Notification::ApplicationActivated,
				n if n == NSWorkspaceDidDeactivateApplicationNotification => Notification::ApplicationDeactivated,
				n if n == NSWorkspaceDidTerminateApplicationNotification => Notification::ApplicationTerminated,
				_ => return Err(()),
			})
		}
	}
}

impl WorkspaceObserver {
	pub fn new(on_event: impl Fn(Event) + 'static + Send + Sync) -> Result<ObserverGuard<Self>, Error> {
		let notification_center = NSWorkspace::sharedWorkspace().notificationCenter();

		Ok(ObserverGuard {
			inner: Self {
				notification_center,
				added_notifications: HashMap::new(),
				on_event_block: RcBlock::new(move |notification: NonNull<NSNotification>| {
					let notification = unsafe { notification.as_ref() };

					let Some(ns_running_app) = get_NSRunningApplication_from_notif(notification) else {
						log::warn!("Received workspace notification without NSRunningApplication, skipping");
						return;
					};

					let notification_name = notification.name();

					let notification = match Notification::try_from(notification_name.as_ref()) {
						Ok(notification) => notification,
						Err(()) => {
							log::warn!("Received workspace notification with invalid notification name, skipping");
							return;
						}
					};

					let event: Event = match notification {
						Notification::ApplicationActivated => Event::ApplicationActivated(ns_running_app),
						Notification::ApplicationDeactivated => Event::ApplicationDeactivated(ns_running_app),
						Notification::ApplicationTerminated => Event::ApplicationTerminated(ns_running_app),
					};

					on_event.call((event,));
				}),
			},
		})
	}
}

impl ObserverGuard<WorkspaceObserver> {
	pub fn add_notification(&mut self, notification: Notification) -> Result<&mut Self, Error> {
		self.inner.added_notifications.insert(notification.clone(), self.inner.on_event_block.clone());
		Ok(self)
	}
}

impl Observer for WorkspaceObserver {
	type Error = Error;
	type Event = Event;

	// This means "make sure workspace notifications are being observed".
	// Calling it twice must not register the same notification twice.
	fn listen(&self) -> Result<(), Self::Error> {
		for (notification_name, on_event_block) in &self.added_notifications {
			unsafe {
				self.notification_center.addObserverForName_object_queue_usingBlock(
					Some(notification_name.clone().try_into().map_err(|_| Error::UnsupportedNotification)?),
					None,
					None,
					on_event_block,
				);
			};
		}

		Ok(())
	}

	fn sleep(&self) -> Result<(), Self::Error> {
		todo!()
	}
}

#[allow(non_snake_case)]
fn get_NSRunningApplication_from_notif(notif: &NSNotification) -> Option<objc2::rc::Retained<NSRunningApplication>> {
	let Some(user_info) = notif.userInfo() else {
		log::warn!("Received workspace notification without userInfo, skipping.");
		return None;
	};

	let Some(ns_running_app) = user_info.valueForKey(&NSString::from_str("NSWorkspaceApplicationKey")) else {
		log::warn!("Received workspace notification without NSWorkspaceApplicationKey, skipping.");
		return None;
	};

	let Ok(ns_running_app) = ns_running_app.downcast::<NSRunningApplication>() else {
		log::warn!("Received workspace notification with invalid NSWorkspaceApplicationKey, skipping.");
		return None;
	};

	Some(ns_running_app)
}

pub fn get_running_applications() -> Vec<Retained<NSRunningApplication>> {
	let running_apps = NSWorkspace::sharedWorkspace().runningApplications();
	running_apps.to_vec()
}

pub fn get_frontmost_application() -> Option<Retained<NSRunningApplication>> {
	NSWorkspace::sharedWorkspace().frontmostApplication()
}
