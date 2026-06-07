use std::{
	collections::HashMap,
	sync::{Arc, Mutex, OnceLock},
};

use common::main_thread::MainThread;
use serde_json::Value;

use crate::{
	ax,
	convert::{accessibility_event_to_json, accessibility_value_to_json},
	get_app_element, instruction,
	instruction::Instruction,
	monitor, register_accessibility_observer, register_app,
	when::var::declare_var,
};

use super::{
	Pack, PackHooks, Response,
	proto::{self, HostHandlers, Incoming, Outgoing},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackId {
	pub publisher_domain: String,
	pub pack_name: String,
}

impl PackId {
	pub fn new(publisher_domain: impl Into<String>, pack_name: impl Into<String>) -> Self {
		Self {
			publisher_domain: publisher_domain.into(),
			pack_name: pack_name.into(),
		}
	}
}

/// The orchestrator-core side of pack execution.
///
/// `Host` owns pack lookup and the host API that packs can call. It does not
/// own sockets, child processes, sandbox profiles, or any other transport
/// detail; a launcher turns those into `Incoming` / `Outgoing` messages.
pub struct Host {
	packs: Mutex<HashMap<PackId, Arc<Pack>>>,
}

impl Host {
	pub fn new() -> Self {
		Self {
			packs: Mutex::new(HashMap::new()),
		}
	}

	/// Attach one live pack to the host.
	///
	/// The launcher provides `outgoing`: libinvoke calls it whenever a typed
	/// `Outgoing` message must be delivered to the pack.
	pub fn attach<O>(self: &Arc<Self>, id: PackId, outgoing: O) -> Arc<Pack>
	where
		O: Fn(Outgoing) + Send + Sync + 'static,
	{
		self.attach_with_hooks(id, outgoing, PackHooks::default())
	}

	/// Attach one live pack and report pack-declared state as it arrives.
	///
	/// Invoke.app uses these callbacks to learn which functions and vars the UI
	/// can present. The pack engine still stores the canonical state here.
	pub fn attach_with_hooks<O>(self: &Arc<Self>, id: PackId, outgoing: O, hooks: PackHooks) -> Arc<Pack>
	where
		O: Fn(Outgoing) + Send + Sync + 'static,
	{
		let pack = Pack::build(outgoing, hooks);
		self.packs.lock().unwrap().insert(id, pack.clone());
		pack
	}

	/// Feed one message from a pack into libinvoke.
	///
	/// This is the launcher's inbound seam. `receive` handles both requests from
	/// the pack and responses to previous `Pack::run` calls.
	pub fn receive(self: &Arc<Self>, pack: &Arc<Pack>, incoming: Incoming) {
		match incoming {
			// Pack is requesting something from us.
			Incoming::Request { id, request } => {
				let host = self.clone();
				let pack = pack.clone();
				tokio::spawn(async move {
					let response = host.handle(&pack, request).await;
					if let Some(id) = id {
						pack.respond(id, response);
					}
				});
			}

			// Pack is responding to one of our runs on it.
			Incoming::Response(id, proto::OK, body) => pack.complete(id, Ok(body)),
			Incoming::Response(id, _, body) => pack.complete(id, Err(body.as_str().unwrap_or_default().to_owned())),
		}
	}

	pub fn remove(&self, id: &PackId) -> bool {
		self.packs.lock().unwrap().remove(id).is_some()
	}

	pub fn get(&self, id: &PackId) -> Option<Arc<Pack>> {
		self.packs.lock().unwrap().get(id).cloned()
	}

	async fn handle(&self, pack: &Arc<Pack>, req: HostHandlers) -> Response {
		use HostHandlers::*;

		match req {
			RegisterApp { bundle_identifier } => {
				if let Some(h) = monitor::app_handle_by_bundle_id(&bundle_identifier) {
					return Ok(Value::from(h.0));
				}
				let h = register_app(bundle_identifier);
				_ = register_accessibility_observer(h).await;
				Ok(Value::from(h))
			}

			DefineFunction {
				app,
				function_name: name,
				view,
			} => {
				let handle = pack.define_function(app, name, view);
				Ok(Value::from(handle.0))
			}

			DeclareVar { name } => {
				let h = declare_var();
				pack.declare_var(name, h);
				Ok(Value::from(h))
			}

			SetVar { var, value } => {
				pack.set_var(var, value);
				Ok(Value::Null)
			}

			RunFunction {
				publisher_domain,
				pack_name,
				function_name,
				payload,
			} => {
				let id = PackId::new(publisher_domain, pack_name);
				let Some(pack) = self.get(&id) else {
					return Err(format!("pack not mounted: {}/{}", id.publisher_domain, id.pack_name));
				};
				pack.run_function(&function_name, payload).await
			}

			Ready {} => {
				pack.mark_ready();
				Ok(Value::Null)
			}

			KeyboardKeyDown { app, key, modifiers } => {
				let mut instruction = instruction::Keyboard::KeyDown { key, modifiers, app };
				MainThread::run(move || instruction.run())
					.await
					.inspect_err(|e| log::error!("Failed to run pack keyboard key down: {e:?}"))
					.map_err(|e| format!("{e:?}"))
					.map(Into::into)
			}
			KeyboardKeyUp { app, key, modifiers } => {
				let mut instruction = instruction::Keyboard::KeyUp { key, modifiers, app };
				MainThread::run(move || instruction.run())
					.await
					.inspect_err(|e| log::error!("Failed to run pack keyboard key up: {e:?}"))
					.map_err(|e| format!("{e:?}"))
					.map(Into::into)
			}
			KeyboardKeyPress { app, key, modifiers } => {
				let mut instruction = instruction::Keyboard::KeyPress { key, modifiers, app };
				MainThread::run(move || instruction.run())
					.await
					.inspect_err(|e| log::error!("Failed to run pack keyboard key press: {e:?}"))
					.map_err(|e| format!("{e:?}"))
					.map(Into::into)
			}
			ScrollWheelY { app, delta } => {
				let mut instruction = instruction::Mouse::ScrollY { delta, app };
				MainThread::run(move || instruction.run())
					.await
					.inspect_err(|e| log::error!("Failed to run pack scroll y: {e:?}"))
					.map_err(|e| format!("{e:?}"))
					.map(Into::into)
			}
			ScrollWheelX { app, delta } => {
				let mut instruction = instruction::Mouse::ScrollX { delta, app };
				MainThread::run(move || instruction.run())
					.await
					.inspect_err(|e| log::error!("Failed to run pack scroll x: {e:?}"))
					.map_err(|e| format!("{e:?}"))
					.map(Into::into)
			}

			GetAppElement { app } => Ok(Value::from(get_app_element(app.0).await)),

			WalkElement { root, filter_path } => {
				let h = MainThread::run(move || instruction::Walk { root, path: filter_path }.run().map(|el| el.map(|el| el.retain())))
					.await
					.map_err(|e| format!("{e:?}"))?;
				if let Some(h) = h {
					pack.retain_element(h);
				}
				Ok(Value::from(h.map(u32::from)))
			}

			DisposeElement { element } => {
				pack.dispose_element(element);
				Ok(Value::Null)
			}

			PerformElementAction { element, action } => MainThread::run(move || instruction::RunAction { element, action }.run())
				.await
				.map_err(|e| format!("{e:?}"))
				.map(Into::into),

			GetElementAttribute { element, attribute } => MainThread::run(move || {
				instruction::GetAttribute {
					element,
					attribute,
					allow_cached: true,
				}
				.run()
				.map(|v| accessibility_value_to_json(&v))
			})
			.await
			.map_err(|e| format!("{e:?}")),

			SetElementAttribute { element, attribute, value } => MainThread::run(move || instruction::SetAttribute { element, attribute, value }.run())
				.await
				.map_err(|e| format!("{e:?}"))
				.map(Into::into),

			ObserveElementNotification { element, notification_name } => {
				let weak = Arc::downgrade(pack);
				let slot = Arc::new(OnceLock::<u32>::new());
				let slot_cb = slot.clone();
				let h = MainThread::run(move || {
					let handle = ax::observe_element_notification(element, notification_name, move |event| {
						let Some(&slot_handle) = slot_cb.get() else { return };
						let Some(pack) = weak.upgrade() else { return };
						pack.dispatch(proto::AccessibilityNotification {
							notification: slot_handle,
							event: accessibility_event_to_json(event),
						});
					})?;
					_ = slot.set(handle);
					Ok::<_, ax::Error>(handle)
				})
				.await
				.map_err(|e| format!("{e:?}"))?;
				pack.retain_notification(h);
				Ok(Value::from(h))
			}

			UnobserveElementNotification { notification } => {
				pack.dispose_notification(notification);
				Ok(Value::Null)
			}
		}
	}
}

impl Default for Host {
	fn default() -> Self {
		Self::new()
	}
}
