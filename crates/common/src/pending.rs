use std::{
	collections::HashMap,
	sync::{
		Mutex,
		atomic::{AtomicU32, Ordering},
	},
};

use tokio::sync::oneshot;

/// Requests we've sent, each awaiting its response. `issue` reserves an id,
/// hands it to the caller to send the request, then waits for `complete`.
///
/// `Mutex<HashMap>`, not `papaya`: a `oneshot::Sender` is consumed when it
/// fires, so removal has to yield it by value.
pub struct Pending<T> {
	next: AtomicU32,
	waiting: Mutex<HashMap<u32, oneshot::Sender<T>>>,
}

impl<T> Pending<T> {
	pub fn new() -> Self {
		Self {
			next: AtomicU32::new(0),
			waiting: Mutex::new(HashMap::new()),
		}
	}

	/// Reserve an id, hand it to `write` to put the request on the wire, then
	/// resolve when the response with that id arrives. If the table is dropped
	/// first, the waiter resolves to `Err(())`.
	pub async fn issue(&self, write: impl FnOnce(u32)) -> Result<T, ()> {
		let id = self.next.fetch_add(1, Ordering::Relaxed);
		let (sender, receiver) = oneshot::channel();
		self.waiting.lock().unwrap().insert(id, sender);
		write(id);
		receiver.await.map_err(|_| ())
	}

	/// Deliver a response to whoever is awaiting `id`.
	pub fn complete(&self, id: u32, value: T) {
		if let Some(sender) = self.waiting.lock().unwrap().remove(&id) {
			_ = sender.send(value);
		}
	}
}

impl<T> Drop for Pending<T> {
	fn drop(&mut self) {
		let waiting = match self.waiting.get_mut() {
			Ok(waiting) => waiting,
			Err(poisoned) => poisoned.into_inner(),
		};

		waiting.clear();
	}
}

impl<T> Default for Pending<T> {
	fn default() -> Self {
		Self::new()
	}
}
