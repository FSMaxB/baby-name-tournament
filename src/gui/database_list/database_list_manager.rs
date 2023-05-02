use crate::gui::backend::Backend;
use crate::gui::database_list::DatabaseView;
use glib::BoxedAnyObject;
use libadwaita::glib;
use once_cell::unsync;
use static_assertions::assert_obj_safe;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub struct DatabaseListManager<View: DatabaseView> {
	filter: Rc<Cell<View::Filter>>,
	count: Rc<Cell<u32>>,
	callback: Rc<unsync::OnceCell<Box<dyn Fn(u32, u32)>>>,
	view: View,
}

impl<View: DatabaseView> DatabaseListManager<View> {
	pub fn new(initial_filter: View::Filter, view: View) -> Self {
		Self {
			filter: Rc::new(Cell::new(initial_filter)),
			count: Default::default(),
			callback: Default::default(),
			view,
		}
	}

	pub fn update_filter(&self, backend: &Backend, filter: View::Filter) {
		self.filter.set(filter);
		self.notify_changed(backend);
	}

	pub fn notify_changed(&self, backend: &Backend) {
		let previous_count = self.count.get();
		let count = self.view.count(backend, self.filter.get());
		if let Some(callback) = self.callback.get() {
			callback(previous_count, count);
		}
	}

	pub(super) fn count(&self, backend: &Backend) -> u32 {
		let count = self.view.count(backend, self.filter.get());
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
	fn read_at_offset(&self, backend: &Backend, offset: u32) -> anyhow::Result<BoxedAnyObject>;
	fn count(&self, backend: &Backend) -> u32;
}

assert_obj_safe!(DynamicListManager);

impl<View: DatabaseView> DynamicListManager for DatabaseListManager<View> {
	fn read_at_offset(&self, backend: &Backend, offset: u32) -> anyhow::Result<BoxedAnyObject> {
		let object = self.view.read_at_offset(backend, self.filter.get(), offset)?;
		Ok(BoxedAnyObject::new(object))
	}

	fn count(&self, backend: &Backend) -> u32 {
		self.count(backend)
	}
}
