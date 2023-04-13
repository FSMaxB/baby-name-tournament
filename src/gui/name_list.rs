use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list_model::DatabaseView;
use crate::gui::name_model::NameModel;
use async_trait::async_trait;
use libadwaita::glib::Object;
use libadwaita::subclass::prelude::*;
use libadwaita::{glib, gtk};
use sqlx::SqlitePool;

mod implementation;

glib::wrapper! {
	pub struct NameList(ObjectSubclass<implementation::NameList>)
		@extends gtk::Widget, gtk::Box,
		@implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for NameList {
	fn default() -> Self {
		Object::new()
	}
}

impl NameList {
	pub fn initialize(&self, backend: Backend) {
		self.imp().initialize(backend, NameListView);
	}
}

struct NameListView;

#[async_trait]
impl DatabaseView for NameListView {
	type RustModel = Name;
	type GObjectModel = NameModel;

	async fn read_at_offset(database_pool: &SqlitePool, offset: u32) -> anyhow::Result<Self::RustModel> {
		Ok(database::read_name_at_offset(offset, Gender::Both, database_pool).await?)
	}

	async fn count(database_pool: &SqlitePool) -> u32 {
		database::count_names(Gender::Both, database_pool)
			.await
			.expect("Faile to count names") as u32
	}
}
