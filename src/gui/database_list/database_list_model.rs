use crate::gui::backend::Backend;
use crate::gui::database_list::database_list_manager::DatabaseListManager;
use crate::gui::database_list::DatabaseView;
use glib::Object;
use libadwaita::subclass::prelude::*;
use libadwaita::{gio, glib};

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
	pub fn new(backend: Backend, database_list_manager: DatabaseListManager<impl DatabaseView>) -> Self {
		let this = Object::new::<Self>();

		this.initialize(backend, database_list_manager);

		this
	}

	pub fn initialize(&self, backend: Backend, database_list_manager: DatabaseListManager<impl DatabaseView>) {
		self.imp().initialize(backend, database_list_manager)
	}
}
