use std::{
	future::Future,
	sync::{
		Arc,
		atomic::{AtomicU32, Ordering},
	},
};

use common::{accessibility::ElementHandle, pending::Pending};
use observer::accessibility::NotificationRegistrationHandle;
use serde_json::Value;
use tokio::sync::watch;

use crate::{
	monitor::AppHandle,
	resource::{Element, Notification, Var},
	when::{self, VarHandle},
};

use super::{
	Response,
	proto::{self, Outgoing},
};

/// A function the pack registered. `view` is opaque to libinvoke — it is just
/// the pack-runtime handle we forward back with `render_view`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
	pub handle: proto::Function,
	pub name: String,
	pub view: proto::View,
}

#[derive(Debug)]
pub enum Error {
	Closed,
	Remote(String),
	Decode(serde_json::Error),
}

#[derive(Default)]
pub struct PackHooks {
	pub function_defined: Option<Box<dyn Fn(&Function) + Send + Sync>>,
	pub var_declared: Option<Box<dyn Fn(&str, VarHandle) + Send + Sync>>,
	pub var_set: Option<Box<dyn Fn(VarHandle, bool) + Send + Sync>>,
}

/// A live pack connection: the outgoing message channel and the `Pending` run
/// table, plus the resource tables the pack has open. Each resource table is
/// keyed by handle, and its values are guards that free the resource on drop,
/// so dropping the `Pack` tears down everything it held.
pub struct Pack {
	// The adapter owns transport; we only emit typed protocol messages.
	outbound: Box<dyn Fn(Outgoing) + Send + Sync>,
	// Our runs on the pack, awaiting their replies.
	pending: Pending<Response>,
	next_function: AtomicU32,
	// Pure ownership tracking — so the elements a pack created get disposed when
	// the pack drops. The element behind a handle lives in the global `ELEMENTS`.
	elements: papaya::HashMap<ElementHandle, Element>,
	vars: papaya::HashMap<String, Var>,
	notifications: papaya::HashMap<NotificationRegistrationHandle, Notification>,
	functions: papaya::HashMap<String, Function>,
	/// The apps this pack registered.
	///
	/// This is solely for whoever embeds libinvoke; libinvoke makes no use of `apps`.
	/// For example, The UI bundle of invoke (Invoke.app) uses this to ensure that shortcuts
	/// are only triggered when one of the apps this pack registered is focused.
	apps: papaya::HashMap<AppHandle, ()>,
	hooks: PackHooks,
	// Becomes `true` once the pack sends `Ready` (initial load + function registration done)
	ready: watch::Sender<bool>,
}

impl Pack {
	pub(super) fn build<O>(outgoing: O, hooks: PackHooks) -> Arc<Self>
	where
		O: Fn(Outgoing) + Send + Sync + 'static,
	{
		Arc::new(Pack {
			outbound: Box::new(outgoing),
			pending: Pending::new(),
			next_function: AtomicU32::new(0),
			elements: papaya::HashMap::new(),
			vars: papaya::HashMap::new(),
			notifications: papaya::HashMap::new(),
			functions: papaya::HashMap::new(),
			apps: papaya::HashMap::new(),
			hooks,
			ready: watch::Sender::new(false),
		})
	}

	/// Run a request on the pack and await its typed response.
	pub async fn request<R, T>(&self, request: R) -> Result<T, Error>
	where
		R: proto::Request<T>,
		T: serde::de::DeserializeOwned,
	{
		let request = request.request();
		let value = self
			.pending
			.issue(|id| self.write(Outgoing::Request { id: Some(id), request }))
			.await
			.map_err(|()| Error::Closed)?
			.map_err(Error::Remote)?;

		serde_json::from_value(value).map_err(Error::Decode)
	}

	/// Dispatch a fire-and-forget event to the pack.
	pub fn dispatch(&self, request: impl proto::Request<()>) {
		self.write(Outgoing::Request {
			id: None,
			request: request.request(),
		});
	}

	pub(super) fn complete(&self, id: u32, response: Response) {
		self.pending.complete(id, response);
	}

	/// Answer one inbound request.
	pub(super) fn respond(&self, id: u32, response: Response) {
		let (status, body) = match response {
			Ok(value) => (proto::OK, value),
			Err(error) => (proto::ERROR, Value::String(error)),
		};
		self.write(Outgoing::Response(id, status, body));
	}

	/// Hand one outbound message to the adapter.
	fn write(&self, message: Outgoing) {
		(self.outbound)(message);
	}

	pub(super) fn define_function(&self, name: String, view: proto::View) -> proto::Function {
		let handle = proto::Function(self.next_function.fetch_add(1, Ordering::Relaxed));
		let function = Function { handle, name, view };
		self.functions.pin().insert(function.name.clone(), function.clone());
		if let Some(hook) = &self.hooks.function_defined {
			hook(&function);
		}
		handle
	}

	/// Record that the pack registered an app. The set scopes the pack's hotkeys.
	pub(super) fn register_app(&self, app: AppHandle) {
		self.apps.pin().insert(app, ());
	}

	/// The apps this pack registered. The orchestrator's shortcut scope iterates
	/// these on every reconcile, so it sees the current set, not a snapshot.
	pub fn apps(&self) -> impl Iterator<Item = AppHandle> {
		self.apps.pin().iter().map(|(&app, ())| app).collect::<Vec<_>>().into_iter()
	}

	pub(super) fn declare_var(&self, name: String, handle: VarHandle) {
		self.vars.pin().insert(name.clone(), Var(handle));
		if let Some(hook) = &self.hooks.var_declared {
			hook(&name, handle);
		}
	}

	pub(super) fn set_var(&self, handle: VarHandle, value: bool) {
		when::var::set_var(handle, value);
		if let Some(hook) = &self.hooks.var_set {
			hook(handle, value);
		}
	}

	pub(super) fn retain_element(&self, element: ElementHandle) {
		self.elements.pin().insert(element, Element(element));
	}

	pub(super) fn dispose_element(&self, element: ElementHandle) {
		self.elements.pin().remove(&element);
	}

	pub(super) fn retain_notification(&self, notification: NotificationRegistrationHandle) {
		self.notifications.pin().insert(notification, Notification(notification));
	}

	pub(super) fn dispose_notification(&self, notification: NotificationRegistrationHandle) {
		self.notifications.pin().remove(&notification);
	}

	#[cfg(test)]
	pub(super) fn function(&self, name: &str) -> Option<Function> {
		self.functions.pin().get(name).cloned()
	}

	/// Mark the pack ready (it sent `Ready`). Idempotent; safe to call with no
	/// waiters — `send_replace` retains the value for whoever subscribes later.
	pub fn mark_ready(&self) {
		self.ready.send_replace(true);
	}

	/// Resolve once the pack has signaled `Ready`. Returns immediately if it
	/// already has. `wait_for` checks the current value before parking, so a
	/// signal that landed before this call is never missed.
	pub async fn ready(&self) -> Result<(), watch::error::RecvError> {
		self.ready.subscribe().wait_for(|&ready| ready).await?;
		Ok(())
	}

	pub async fn run_function(&self, function_name: &str, payload: Value) -> Response {
		let function = self
			.functions
			.pin()
			.get(function_name)
			.map(|function| function.handle)
			.ok_or_else(|| format!("FunctionNotFound:{function_name}"))?;
		proto::RunFunction { function, payload }.request(self).await.map_err(|e| format!("{e:?}"))
	}

	pub async fn render_view(&self, view: proto::View) -> Result<Value, Error> {
		proto::RenderView { view }.request(self).await
	}

	pub async fn run_view_action(&self, view: proto::View, action_id: impl Into<String>, args: Value) -> Result<Value, Error> {
		proto::RunViewAction {
			action_id: action_id.into(),
			args,
			view,
		}
		.request(self)
		.await
	}
}

pub trait RunPack<Response>: proto::Request<Response> + Sized
where
	Response: serde::de::DeserializeOwned,
{
	fn request<'a>(self, pack: &'a Pack) -> impl Future<Output = Result<Response, Error>> + 'a
	where
		Self: 'a,
		Response: 'a,
	{
		pack.request(self)
	}
}

impl<R, Response> RunPack<Response> for R
where
	R: proto::Request<Response>,
	Response: serde::de::DeserializeOwned,
{
}
