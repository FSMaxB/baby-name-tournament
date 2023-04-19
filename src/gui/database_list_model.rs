use crate::gui::backend::Backend;
use glib::Object;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;
use libadwaita::{gio, glib};
use once_cell::unsync;
use static_assertions::assert_obj_safe;
use std::any::{Any, TypeId};
use std::cell::Cell;

mod implementation;

glib::wrapper! {
	pub struct DatabaseListModel(ObjectSubclass<implementation::DatabaseListModel>)
		@implements gio::ListModel
	;
}

impl Default for DatabaseListModel {
	fn default() -> Self {
		Object::new()
	}
}

impl DatabaseListModel {
	pub fn new(backend: Backend, database_view: impl DatabaseView) -> Self {
		let this = Object::new::<Self>();

		this.initialize(backend, database_view);

		this
	}

	pub fn initialize(&self, backend: Backend, database_view: impl DatabaseView) {
		self.imp().initialize(backend, database_view)
	}
}

pub trait DatabaseView: 'static {
	type RustModel;
	type Filter;
	type GObjectModel: StaticType + ObjectType + IsA<Object> + From<Self::RustModel>;

	fn read_at_offset(&self, backend: &Backend, offset: u32) -> anyhow::Result<Self::RustModel>;
	fn count(&self, backend: &Backend) -> u32;
	fn update_filter(&self, backend: &Backend, filter: Self::Filter);

	fn items_changed_callback_storage(&self) -> &unsync::OnceCell<Box<dyn Fn(u32, u32)>>;
	fn count_storage(&self) -> &Cell<u32>;
}

pub trait DatabaseViewExt: DatabaseView {
	fn notify_changed(&self, backend: &Backend) {
		let previous_count = self.count_storage().get();
		let count = self.count(backend);
		if let Some(callback) = self.items_changed_callback_storage().get() {
			callback(previous_count, count);
		}
	}
}

impl<T: DatabaseView> DatabaseViewExt for T {}

trait DynamicDatabaseView {
	fn rust_model(&self) -> TypeId;
	fn gobject_model(&self) -> glib::Type;

	fn update_filter(&self, backend: &Backend, filter: Box<dyn Any>);
	fn read_at_offset(&self, backend: &Backend, offset: u32) -> anyhow::Result<Box<dyn Any>>;
	fn count(&self, backend: &Backend) -> u32;

	fn convert(&self, rust_model: Box<dyn Any>) -> Object;

	fn register_items_changed_callback(&self, callback: Box<dyn Fn(u32, u32)>);
}

impl<T: DatabaseView> DynamicDatabaseView for T {
	fn rust_model(&self) -> TypeId {
		TypeId::of::<T::RustModel>()
	}

	fn gobject_model(&self) -> glib::Type {
		T::GObjectModel::static_type()
	}

	fn read_at_offset(&self, backend: &Backend, offset: u32) -> anyhow::Result<Box<dyn Any>> {
		let model = T::read_at_offset(self, backend, offset)?;
		Ok(Box::new(model))
	}

	fn count(&self, backend: &Backend) -> u32 {
		let count = T::count(self, backend);
		self.count_storage().set(count);

		count
	}

	fn update_filter(&self, backend: &Backend, filter: Box<dyn Any>) {
		let filter = *filter.downcast::<T::Filter>().expect("Incorrect filter type");
		T::update_filter(self, backend, filter);
	}

	fn convert(&self, rust_model: Box<dyn Any>) -> Object {
		let rust_model = *rust_model.downcast::<T::RustModel>().expect("Incorrect type");
		let gobject_model = T::GObjectModel::from(rust_model);
		gobject_model.upcast()
	}

	fn register_items_changed_callback(&self, callback: Box<dyn Fn(u32, u32)>) {
		self.items_changed_callback_storage()
			.set(callback)
			.unwrap_or_else(|_| panic!("Callback was already set"));
	}
}

assert_obj_safe!(DynamicDatabaseView);
