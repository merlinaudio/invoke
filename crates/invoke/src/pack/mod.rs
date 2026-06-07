//! A Pack is an attached connection to pack code. The link is symmetric: either
//! side can send the other a request and get a response.
//!
//! A launcher is deliberately small. It owns transport, process lifetime, and
//! platform policy, then adapts one live pack connection to the pack engine:
//!
//! 1. create or connect to whatever runs the pack;
//! 2. call `Host::attach` / `Host::attach_with_hooks` with a `PackId` and an
//!    outgoing message emitter;
//! 3. send every emitted `proto::Outgoing` message to the pack;
//! 4. feed every `proto::Incoming` message from the pack into `Host::receive`;
//! 5. stop exposing the pack through `Host` when the transport ends, then let
//!    ownership drop the pack state.
//!
//! Mount policy sits one level above that. The orchestrator that owns the mount table
//! decides when a pack is mounted or unmounted, stores whatever guard the
//! launcher returns, and calls `Host::remove` when it unmounts the pack.
//!
//! The wire vocabulary lives in `proto`, but the transport does not. A launcher
//! can put those messages on a Unix socket, pipe, websocket, remote server, test
//! channel, or something else without changing the pack engine. We reach the
//! pack with `pack.run(req).await` / `req.run(pack).await` for typed responses,
//! or `pack.dispatch(req)` for fire-and-forget events.

mod host;
mod pack;
pub mod proto;

use serde_json::Value;

pub use host::{Host, PackId};
pub use pack::{Error, Function, Pack, PackHooks, RunPack};

/// The result of handling an inbound request: a JSON value, or an error message.
pub type Response = Result<Value, String>;

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::{Arc, Mutex};

	use crate::monitor::AppHandle;
	use crate::when::VarHandle;
	use crate::when::var::get_var;
	use proto::{HostHandlers, Incoming, Outgoing};
	use tokio::sync::mpsc;

	fn test_outgoing() -> (impl Fn(Outgoing) + Send + Sync + 'static, mpsc::UnboundedReceiver<Outgoing>) {
		let (outgoing, outgoing_receiver) = mpsc::unbounded_channel();
		let outgoing = move |message| {
			_ = outgoing.send(message);
		};
		(outgoing, outgoing_receiver)
	}

	fn test_host() -> Arc<Host> {
		Arc::new(Host::new())
	}

	async fn request(host: &Arc<Host>, pack: &Arc<Pack>, outgoing: &mut mpsc::UnboundedReceiver<Outgoing>, request: HostHandlers) -> Response {
		host.receive(pack, Incoming::Request { id: Some(1), request });

		let Some(Outgoing::Response(_, status, body)) = outgoing.recv().await else {
			panic!("expected a response");
		};

		match status {
			proto::OK => Ok(body),
			_ => Err(body.as_str().unwrap_or_default().to_owned()),
		}
	}

	#[test]
	fn host_handlers_wire_format() {
		let req = HostHandlers::DeclareVar { name: "test".into() };
		let json = serde_json::to_string(&req).unwrap();
		assert_eq!(json, r#"{"declareVar":{"name":"test"}}"#);
		let back: HostHandlers = serde_json::from_str(&json).unwrap();
		assert!(matches!(back, HostHandlers::DeclareVar { name } if name == "test"));

		let req = HostHandlers::RunFunction {
			publisher_domain: "getinvoke.com".into(),
			pack_name: "abletonlive".into(),
			function_name: "zoomIn".into(),
			payload: Value::String("null".into()),
		};
		assert_eq!(
			serde_json::to_string(&req).unwrap(),
			r#"{"runFunction":{"publisherDomain":"getinvoke.com","packName":"abletonlive","functionName":"zoomIn","payload":"null"}}"#
		);

		let req: HostHandlers = serde_json::from_str(r#"{"observeElementNotification":{"element":1,"notificationName":"AXFocusedUIElementChanged"}}"#).unwrap();
		assert!(matches!(
			req,
			HostHandlers::ObserveElementNotification {
				element,
				notification_name: common::accessibility::Notification::FocusedUIElementChanged,
			} if element == 1.into()
		));
	}

	#[test]
	fn pack_handlers_wire_format() {
		let req = proto::PackHandlers::RunFunction(proto::RunFunction {
			function: proto::Function(42),
			payload: Value::Null,
		});
		assert_eq!(serde_json::to_string(&req).unwrap(), r#"{"runFunction":{"function":42,"payload":null}}"#,);
	}

	#[test]
	fn frame_wire_format() {
		let request = serde_json::to_string(&Incoming::Request {
			id: Some(7),
			request: HostHandlers::DeclareVar { name: "x".into() },
		})
		.unwrap();
		assert_eq!(request, r#"{"id":7,"declareVar":{"name":"x"}}"#);

		let response = serde_json::to_string(&Outgoing::Response(7, proto::OK, Value::from(42))).unwrap();
		assert_eq!(response, r#"[7,0,42]"#);

		// The flattened externally-tagged enum round-trips back.
		let back: Incoming = serde_json::from_str(&request).unwrap();
		assert!(matches!(back, Incoming::Request { id: Some(7), request: HostHandlers::DeclareVar { name } } if name == "x"));
	}

	#[tokio::test(flavor = "current_thread")]
	async fn receive_declare_var() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let pack = host.attach(PackId::new("getinvoke.com", "test"), outgoing);
		let result = request(&host, &pack, &mut outgoing_receiver, HostHandlers::DeclareVar { name: "myvar".into() }).await;
		let _: VarHandle = serde_json::from_value(result.unwrap()).unwrap();
	}

	#[tokio::test(flavor = "current_thread")]
	async fn receive_set_var() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let pack = host.attach(PackId::new("getinvoke.com", "test"), outgoing);
		let result = request(&host, &pack, &mut outgoing_receiver, HostHandlers::DeclareVar { name: "v".into() })
			.await
			.unwrap();
		let handle: VarHandle = serde_json::from_value(result).unwrap();
		request(&host, &pack, &mut outgoing_receiver, HostHandlers::SetVar { var: handle, value: true })
			.await
			.unwrap();
		assert!(get_var(handle));
	}

	#[tokio::test(flavor = "current_thread")]
	async fn receive_define_function() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let pack = host.attach(PackId::new("getinvoke.com", "test"), outgoing);
		let result = request(
			&host,
			&pack,
			&mut outgoing_receiver,
			HostHandlers::DefineFunction {
				app: Some(AppHandle(9)),
				function_name: "zoom".into(),
				view: proto::View(11),
			},
		)
		.await;
		let handle: u32 = serde_json::from_value(result.unwrap()).unwrap();

		let function = pack.function("zoom").unwrap();
		assert_eq!(function.handle, proto::Function(handle));
		assert_eq!(function.app, Some(AppHandle(9)));
		assert_eq!(function.name, "zoom");
		assert_eq!(function.view, proto::View(11));
	}

	#[tokio::test(flavor = "current_thread")]
	async fn receive_run_function_uses_registry() {
		let host = test_host();
		let (source_outgoing, mut source_outgoing_receiver) = test_outgoing();
		let source = host.attach(PackId::new("getinvoke.com", "source"), source_outgoing);
		let (target_outgoing, mut target_outgoing_receiver) = test_outgoing();
		let target = host.attach(PackId::new("getinvoke.com", "abletonlive"), target_outgoing);
		let target_host = host.clone();
		let target_pack = target.clone();

		let function_result = request(
			&host,
			&target,
			&mut target_outgoing_receiver,
			HostHandlers::DefineFunction {
				app: None,
				function_name: "zoomIn".into(),
				view: proto::View(1),
			},
		)
		.await
		.unwrap();
		let function_handle: u32 = serde_json::from_value(function_result).unwrap();

		let responder = tokio::spawn(async move {
			let outgoing = target_outgoing_receiver.recv().await.unwrap();
			let Outgoing::Request {
				id: Some(id),
				request: proto::PackHandlers::RunFunction(proto::RunFunction { function, payload }),
			} = outgoing
			else {
				panic!("expected runFunction request");
			};
			assert_eq!(function, proto::Function(function_handle));
			assert_eq!(payload, Value::String("null".into()));
			target_host.receive(&target_pack, Incoming::Response(id, proto::OK, serde_json::json!("done")));
		});

		let result = request(
			&host,
			&source,
			&mut source_outgoing_receiver,
			HostHandlers::RunFunction {
				publisher_domain: "getinvoke.com".into(),
				pack_name: "abletonlive".into(),
				function_name: "zoomIn".into(),
				payload: Value::String("null".into()),
			},
		)
		.await
		.unwrap();
		assert_eq!(result, serde_json::json!("done"));
		responder.await.unwrap();
	}

	#[tokio::test(flavor = "current_thread")]
	async fn hooks_report_registered_state() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let functions = Arc::new(Mutex::new(Vec::<Function>::new()));
		let vars = Arc::new(Mutex::new(Vec::<(String, VarHandle)>::new()));
		let var_sets = Arc::new(Mutex::new(Vec::<(VarHandle, bool)>::new()));

		let pack = host.attach_with_hooks(
			PackId::new("getinvoke.com", "test"),
			outgoing,
			PackHooks {
				function_defined: Some(Box::new({
					let functions = functions.clone();
					move |function| functions.lock().unwrap().push(function.clone())
				})),
				var_declared: Some(Box::new({
					let vars = vars.clone();
					move |name, handle| vars.lock().unwrap().push((name.to_owned(), handle))
				})),
				var_set: Some(Box::new({
					let var_sets = var_sets.clone();
					move |handle, value| var_sets.lock().unwrap().push((handle, value))
				})),
			},
		);

		let var_result = request(&host, &pack, &mut outgoing_receiver, HostHandlers::DeclareVar { name: "ready".into() })
			.await
			.unwrap();
		let var_handle: VarHandle = serde_json::from_value(var_result).unwrap();

		let function_result = request(
			&host,
			&pack,
			&mut outgoing_receiver,
			HostHandlers::DefineFunction {
				app: Some(AppHandle(2)),
				function_name: "save".into(),
				view: proto::View(8),
			},
		)
		.await
		.unwrap();
		let function_handle: u32 = serde_json::from_value(function_result).unwrap();

		assert_eq!(&*vars.lock().unwrap(), &[("ready".into(), var_handle)]);
		request(&host, &pack, &mut outgoing_receiver, HostHandlers::SetVar { var: var_handle, value: true })
			.await
			.unwrap();
		assert_eq!(&*var_sets.lock().unwrap(), &[(var_handle, true)]);
		assert_eq!(
			&*functions.lock().unwrap(),
			&[Function {
				handle: proto::Function(function_handle),
				app: Some(AppHandle(2)),
				name: "save".into(),
				view: proto::View(8),
			}]
		);
	}

	#[tokio::test(flavor = "current_thread")]
	async fn round_trip_over_connection() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let pack = host.attach(PackId::new("getinvoke.com", "test"), outgoing);

		// The pack sends a request; we expect a response echoing its id.
		host.receive(
			&pack,
			Incoming::Request {
				id: Some(1),
				request: HostHandlers::DeclareVar { name: "roundtrip".into() },
			},
		);

		let Some(Outgoing::Response(id, status, body)) = outgoing_receiver.recv().await else {
			panic!("expected a response");
		};

		assert_eq!(id, 1);
		assert_eq!(status, proto::OK);
		let _: VarHandle = serde_json::from_value(body).unwrap();
	}

	#[tokio::test(flavor = "current_thread")]
	async fn pack_run_round_trip() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let pack = host.attach(PackId::new("getinvoke.com", "test"), outgoing);
		let responder_host = host.clone();
		let responder_pack = pack.clone();

		// The pack side: read our request, respond to its id with a payload.
		let responder = tokio::spawn(async move {
			let outgoing = outgoing_receiver.recv().await.unwrap();
			let Outgoing::Request {
				id: Some(id),
				request: proto::PackHandlers::RenderView(proto::RenderView { view }),
			} = outgoing
			else {
				panic!("expected renderView request");
			};
			assert_eq!(view, proto::View(7));
			responder_host.receive(&responder_pack, Incoming::Response(id, proto::OK, serde_json::json!("flight")));
		});

		let result = pack.render_view(proto::View(7)).await.unwrap();
		assert_eq!(result, serde_json::json!("flight"));
		responder.await.unwrap();
	}

	#[tokio::test(flavor = "current_thread")]
	async fn run_view_action_round_trip() {
		let host = test_host();
		let (outgoing, mut outgoing_receiver) = test_outgoing();
		let pack = host.attach(PackId::new("getinvoke.com", "test"), outgoing);
		let responder_host = host.clone();
		let responder_pack = pack.clone();

		let responder = tokio::spawn(async move {
			let outgoing = outgoing_receiver.recv().await.unwrap();
			let Outgoing::Request {
				id: Some(id),
				request: proto::PackHandlers::RunViewAction(proto::RunViewAction { action_id, args, view }),
			} = outgoing
			else {
				panic!("expected runViewAction request");
			};
			assert_eq!(action_id, "save");
			assert_eq!(args, serde_json::json!([1, 2]));
			assert_eq!(view, proto::View(3));
			responder_host.receive(&responder_pack, Incoming::Response(id, proto::OK, serde_json::json!("rerendered")));
		});

		let result = pack.run_view_action(proto::View(3), "save", serde_json::json!([1, 2])).await.unwrap();
		assert_eq!(result, serde_json::json!("rerendered"));
		responder.await.unwrap();
	}

	#[tokio::test(flavor = "current_thread")]
	async fn drop_unmounts() {
		let host = test_host();
		let id = PackId::new("getinvoke.com", "test");
		let (outgoing, outgoing_receiver) = test_outgoing();
		let pack = host.attach(id.clone(), outgoing);
		let weak = Arc::downgrade(&pack);

		host.remove(&id);
		drop(pack);
		drop(outgoing_receiver);

		tokio::task::yield_now().await;
		tokio::task::yield_now().await;

		assert!(weak.upgrade().is_none());
	}
}
