use crate::gui::backend::Backend;
use async_trait::async_trait;
use glib::Object;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;
use libadwaita::{gio, glib};
use sqlx::SqlitePool;
use static_assertions::assert_obj_safe;
use std::any::{Any, TypeId};

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

#[async_trait]
pub trait DatabaseView: Send + Sync + 'static {
	type RustModel: Send;
	type GObjectModel: StaticType + ObjectType + IsA<Object> + From<Self::RustModel>;

	async fn read_at_offset(database_pool: &SqlitePool, offset: u32) -> anyhow::Result<Self::RustModel>;
	async fn count(database_pool: &SqlitePool) -> u32;
}

#[async_trait]
trait DynamicDatabaseView {
	fn rust_model(&self) -> TypeId;
	fn gobject_model(&self) -> glib::Type;

	async fn read_at_offset(&self, database_pool: &SqlitePool, offset: u32) -> anyhow::Result<Box<dyn Any + Send>>;
	async fn count(&self, database_pool: &SqlitePool) -> u32;

	fn convert(&self, rust_model: Box<dyn Any>) -> Object;
}

#[async_trait]
impl<T: DatabaseView> DynamicDatabaseView for T {
	fn rust_model(&self) -> TypeId {
		TypeId::of::<T::RustModel>()
	}

	fn gobject_model(&self) -> glib::Type {
		T::GObjectModel::static_type()
	}

	async fn read_at_offset(&self, database_pool: &SqlitePool, offset: u32) -> anyhow::Result<Box<dyn Any + Send>> {
		let model = T::read_at_offset(database_pool, offset).await?;
		Ok(Box::new(model))
	}

	async fn count(&self, database_pool: &SqlitePool) -> u32 {
		T::count(database_pool).await
	}

	fn convert(&self, rust_model: Box<dyn Any>) -> Object {
		let rust_model = *rust_model.downcast::<T::RustModel>().expect("Incorrect type");
		let gobject_model = T::GObjectModel::from(rust_model);
		gobject_model.upcast()
	}
}

assert_obj_safe!(DynamicDatabaseView);
