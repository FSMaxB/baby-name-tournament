use crate::gui::backend::Backend;
use crate::gui::database_list::{DatabaseView, Model};
use anyhow::Context;
use glib::BoxedAnyObject;
use libadwaita::glib;
use once_cell::unsync;
use static_assertions::assert_obj_safe;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct DatabaseListManager<View: DatabaseView> {
	backend: Backend,
	filter: Rc<RefCell<View::Filter>>,
	// Initially the idea was to lazily read everything from the database
	// directly. The issue is that for some reason, GTKs ListView still eagerly
	// reads every element in the beginning ... 😠
	// This means I probably could have just used a ListStore directly in the beginning
	// but now it's a bit late and I'm not going to touch all the machinery in between for now.
	element_cache: Rc<RefCell<Vec<View::Model>>>,
	callback: Callback,
	view: View,
}

type Callback = Rc<unsync::OnceCell<Box<dyn Fn(u32, u32)>>>;

impl<View: DatabaseView> DatabaseListManager<View> {
	pub fn new(initial_filter: View::Filter, view: View, backend: Backend) -> anyhow::Result<Self> {
		let all_elements = view
			.read_all(&backend, &initial_filter)
			.context("Caching all elements initially")?;
		Ok(Self {
			backend,
			filter: Rc::new(RefCell::new(initial_filter)),
			element_cache: Rc::new(RefCell::new(all_elements)),
			callback: Default::default(),
			view,
		})
	}

	pub fn update_filter(&self, filter: View::Filter) -> anyhow::Result<()> {
		*self.filter.borrow_mut() = filter;
		self.notify_changed()
	}

	pub fn notify_updated(&self, key: &<View::Model as Model>::Key) -> anyhow::Result<()> {
		let updated_element = self.view.read_by_key(&self.backend, key)?;

		let mut element_cache = self.element_cache.borrow_mut();
		let Some(index) = element_cache.iter().position(|element| element.unique_key() == key) else {
			// if it isn't in the list, it can't be updated
			return Ok(())
		};

		element_cache[index] = updated_element;

		if let Some(callback) = self.callback.get() {
			let count = self.count();
			// FIXME: Change callback so that only a single row can be updated in isolation
			callback(count, count);
		}

		Ok(())
	}

	pub fn notify_changed(&self) -> anyhow::Result<()> {
		let previous_count = self.count();
		self.fetch_all()?;
		let count = self.count();

		if let Some(callback) = self.callback.get() {
			callback(previous_count, count);
		}

		Ok(())
	}

	pub fn read_at_offset(&self, offset: u32) -> anyhow::Result<View::Model> {
		self.element_cache
			.borrow()
			.get(offset as usize)
			.cloned()
			.with_context(|| format!("No element at offset {offset}"))
	}

	pub(super) fn count(&self) -> u32 {
		u32::try_from(self.element_cache.borrow().len()).expect("Had more than 2^32 elements. GTK can't handle that")
	}

	pub(super) fn register_items_changed_callback(&self, callback: Box<dyn Fn(u32, u32)>) {
		self.callback
			.set(callback)
			.unwrap_or_else(|_| panic!("Callback was already set"));
	}

	fn fetch_all(&self) -> anyhow::Result<()> {
		let all_elements = self.view.read_all(&self.backend, self.filter.borrow().deref())?;

		*self.element_cache.borrow_mut() = all_elements;
		Ok(())
	}

	pub(super) fn erase(self) -> Box<dyn DynamicListManager> {
		Box::new(self)
	}
}

pub(super) trait DynamicListManager {
	fn read_at_offset(&self, offset: u32) -> anyhow::Result<BoxedAnyObject>;
	fn count(&self) -> u32;
}

assert_obj_safe!(DynamicListManager);

impl<View: DatabaseView> DynamicListManager for DatabaseListManager<View> {
	fn read_at_offset(&self, offset: u32) -> anyhow::Result<BoxedAnyObject> {
		let object = self.read_at_offset(offset)?;
		Ok(BoxedAnyObject::new(object))
	}

	fn count(&self) -> u32 {
		self.count()
	}
}
