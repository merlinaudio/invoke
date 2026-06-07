use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::MainThreadMarker;
use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};
use tokio::sync::oneshot;

/// Wrapper that ensures the inner type can only be accessed on the main thread. Makes a non-Send type Send.
///
/// This is essentially a runtime-checked `Send`, much like e.g. `RefCell` is a runtime-checked `Borrow`,
/// because **the type can't REALLY be sent across threads.**
/// It just makes sure of that at runtime instead of at compile time.
///
/// # Panics
/// Be very aware that **`Deref`ing this type may panic.** This fail-fast behavior is **intended and I argue it's good**:
///
/// - Types that use `MainThread` usually NEVER work off the main thread, crashing fast like this is a fine behavior and signals a bug in the program.
/// - It's very unlikely that if a type is wrapped in `MainThread`, it will be accessed off the main thread in only fringe conditions.
///   
///   Why? Usually, code that uses `MainThread` is wrapped in `MainThread::run` or similar. Everything in that closure is fine, as it's run on the main thread.
///
///   And in cases where main threaded access is implicit, e.g. in a non-async function called from the main thread, it's also fine,
///   because changing the thread context is
///   1. rare (how often do you change a non-async function that uses MainThread to an async one?)
///   2. usually immediately going to cause a panic with this type
///
///   Of course there still is the case of "my async runtime always assigned this async context to my main thread in build X, but not build Y".
///   Frankly, for this project, this isn't a big deal. But **for your project, you may want to consider using a different approach**, or replacing the panicking behavior with Result<T,E>
///
///  
/// So it is assumed that (1) crashes are noticed immediately and (2) it's unlikely that there's a hidden "bad path" that causes a panic in a usually-OK block of code.
///
/// ---
///
/// Using the contained type T off the main thread is usually a bug,
/// and panicking then usually leads to just the async runtime's thread exiting, not the whole program.
///
/// Crashing a thread with a standout error message is usually preferable to leaving in the bug obviously, or, less obviously,
/// accidentally using `?` to silently fail like the value doesn't exist. When it really does exist, but wasn't accessed from the main thread.
#[derive(Debug)]
pub struct MainThread<T>(MainThreadBound<T>);

/// SAFETY: Internally, MainThreadBound takes care of ensuring the type is Send
unsafe impl<T> Send for MainThread<T> {}
/// SAFETY: Internally, MainThreadBound takes care of ensuring the type is Sync
unsafe impl<T> Sync for MainThread<T> {}

impl<T> MainThread<T> {
	pub fn new(t: T) -> Self {
		let mtm = unsafe { MainThreadMarker::new_unchecked() };
		Self(MainThreadBound::new(t, mtm))
	}

	/// Gets the inner value of this container.
	///
	/// # Panics
	/// Panics if called off the main thread.
	///
	/// Despite this behavior, this is usually an OK function to call, because calling it off the main thread is usually a bug,
	/// and in many cases, will lead to just the async runtime's thread exiting, not the whole program.
	/// (And if called from the main thread, it won't panic, obviously.)
	pub fn unwrap(&self) -> &T {
		self.0.get(MainThreadMarker::new().expect("Not on main thread"))
	}

	/// Gets the inner value of this container. Returns `None` if called off the main thread.
	///
	/// # You may want to use `unwrap` instead
	///
	/// Despite the fact that `unwrap()` may panic, you may want to use it instead of this function,
	/// especially if getting the value off the main thread indicates a bug.
	///
	/// A good example of this is a tauri::command which has been declared async, but really needs to run on the main thread.
	/// If you use this function, you have to handle the case where the command is called off the main thread, even though this will never work.
	/// And in such an obvious case, crashing a thread (which will immediately be restarted by most async runtimes like tokio anyways) is preferable to
	/// silently failing due to use of the `?` operator, and less work than logging the error and stopping execution. Such errors are also less likely to be noticed.
	pub fn get(&self) -> Option<&T> {
		Some(self.0.get(MainThreadMarker::new()?))
	}

	pub fn get_mut(&mut self) -> Option<&mut T> {
		Some(self.0.get_mut(MainThreadMarker::new()?))
	}

	pub fn into_inner(self) -> Option<T> {
		Some(self.0.into_inner(MainThreadMarker::new()?))
	}
}

impl<Ret: 'static + Send + Debug> MainThread<Ret> {
	/// Helper function unrelated to the MainThread-wrapped types. Allows running code on the main thread.
	///
	/// If you don't need the return value of `f`, use `MainThread::dispatch` instead.
	///
	/// Internally uses exec_async, which is preferred over `MainThread::run_blocking` (which uses exec_sync), for performance reasons.
	///
	/// Usage:
	///
	/// ```rs
	/// MainThread::run(|| {
	///     // code that must be run on the main thread
	/// });
	/// ```
	pub async fn run(f: impl FnOnce() -> Ret + Send + 'static) -> Ret {
		let (sender, receiver) = oneshot::channel();

		DispatchQueue::main().exec_async(move || {
			let result = f();
			if let Err(e) = sender.send(result) {
				log::error!("Failed to send result to main thread: {e:?}")
			}
		});

		receiver.await.unwrap()
	}
}

impl<Ret: Send> MainThread<Ret> {
	/// Helper function unrelated to the MainThread-wrapped types. Allows running code on the main thread.
	///
	/// This is the same as `MainThread::run`, but uses dispatch2's `exec_sync` instead of `exec_async`, which is less performant.
	/// Therefore, unless you need to block the main thread, use `MainThread::run` instead.
	///
	/// If you need to block the main thread, but don't need the return value of `f`, use `MainThread::dispatch_blocking` instead.
	pub fn run_blocking(f: impl FnOnce() -> Ret + Send) -> Ret {
		let mut ret = None;
		DispatchQueue::main().exec_sync(|| {
			ret = Some(f());
		});
		ret.unwrap()
	}
}

impl MainThread<()> {
	/// Helper function unrelated to the MainThread-wrapped types. Allows running code on the main thread.
	///
	/// This is the same as `MainThread::run`, but doesn't return anything, which is slightly more performant.
	pub fn dispatch(f: impl FnOnce() + Send + 'static) {
		DispatchQueue::main().exec_async(f);
	}

	/// Helper function unrelated to the MainThread-wrapped types. Allows running code on the main thread.
	///
	/// This is the same as `MainThread::dispatch`, but waits for the inner function to return.
	///
	/// **You almost never need this function.** The only use case is when you need to perform a side-effect that must immediately be visible to the main thread after the function has returned.
	///
	/// **You almost always want to use `MainThread::run_blocking` instead**, which will return the value of the inner function at no additional cost.
	pub fn dispatch_blocking(f: impl FnOnce() + Send) {
		DispatchQueue::main().exec_sync(f);
	}
}

impl<T> Deref for MainThread<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.unwrap()
	}
}

impl<T> DerefMut for MainThread<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.get_mut().unwrap()
	}
}

impl<T> From<MainThread<T>> for Option<T> {
	fn from(value: MainThread<T>) -> Self {
		value.into_inner()
	}
}
