use crate::gui::backend::Backend;
use crate::gui::database_list::DatabaseView;
use glib::BoxedAnyObject;
use libadwaita::glib;
use once_cell::unsync;
use static_assertions::assert_obj_safe;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct DatabaseListManager<View: DatabaseView> {
	backend: Backend,
	filter: Rc<RefCell<View::Filter>>,
	count: Rc<Cell<u32>>,
	callback: Callback,
	view: View,
}

type Callback = Rc<unsync::OnceCell<Box<dyn Fn(u32, u32)>>>;

impl<View: DatabaseView> DatabaseListManager<View> {
	pub fn new(initial_filter: View::Filter, view: View, backend: Backend) -> Self {
		Self {
			backend,
			filter: Rc::new(RefCell::new(initial_filter)),
			count: Default::default(),
			callback: Default::default(),
			view,
		}
	}

	pub fn update_filter(&self, filter: View::Filter) {
		*self.filter.borrow_mut() = filter;
		self.notify_changed();
	}

	pub fn notify_changed(&self) {
		let previous_count = self.count.get();
		let count = self.view.count(&self.backend, self.filter.borrow().deref());
		if let Some(callback) = self.callback.get() {
			callback(previous_count, count);
		}
	}

	pub fn read_at_offset(&self, offset: u32) -> anyhow::Result<View::Model> {
		self.view
			.read_at_offset(&self.backend, self.filter.borrow().deref(), offset)
	}

	pub(super) fn count(&self) -> u32 {
		let count = self.view.count(&self.backend, self.filter.borrow().deref());
		self.count.set(count);

		count
	}

	pub(super) fn register_items_changed_callback(&self, callback: Box<dyn Fn(u32, u32)>) {
		self.callback
			.set(callback)
			.unwrap_or_else(|_| panic!("Callback was already set"));
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
