use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	sync::atomic::{AtomicU32, Ordering},
};

#[derive(Debug, Default)]
pub struct ConcurrentHandleMapU32<T> {
	map: papaya::HashMap<u32, T>,
	counter: AtomicU32,
}

impl<T> ConcurrentHandleMapU32<T> {
	pub fn new() -> Self {
		Self {
			map: papaya::HashMap::new(),
			counter: AtomicU32::new(0),
		}
	}

	pub fn insert(&self, value: T) -> u32 {
		let handle = self.next_handle();
		self.map.pin().insert(handle, value);
		handle
	}

	pub fn replace(&self, handle: u32, value: T) {
		self.map.pin().insert(handle, value);
	}

	pub fn next_handle(&self) -> u32 {
		self.counter.fetch_add(1, Ordering::Relaxed)
	}

	pub fn remove(&self, handle: u32) {
		self.map.pin().remove(&handle);
	}
}

impl<T> Deref for ConcurrentHandleMapU32<T> {
	type Target = papaya::HashMap<u32, T>;
	fn deref(&self) -> &Self::Target {
		&self.map
	}
}

impl<T> DerefMut for ConcurrentHandleMapU32<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.map
	}
}

// ---------------------------------------------------------------------------------------------------------------------

pub struct HandleMapU32<T> {
	map: HashMap<u32, T>,
	counter: AtomicU32,
}

impl<T> HandleMapU32<T> {
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),
			counter: AtomicU32::new(0),
		}
	}

	pub fn insert(&mut self, value: T) -> u32 {
		let handle = self.counter.fetch_add(1, Ordering::Relaxed);
		self.map.insert(handle, value);
		handle
	}

	pub fn replace(&mut self, handle: u32, value: T) {
		self.map.insert(handle, value);
	}

	pub fn remove(&mut self, handle: u32) -> Option<T> {
		self.map.remove(&handle)
	}
}

impl<T> Default for HandleMapU32<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Deref for HandleMapU32<T> {
	type Target = HashMap<u32, T>;
	fn deref(&self) -> &Self::Target {
		&self.map
	}
}

impl<T> DerefMut for HandleMapU32<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.map
	}
}
