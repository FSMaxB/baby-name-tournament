use crate::gui::backend::Backend;
use crate::gui::database_list_model::{DatabaseView, DynamicDatabaseView};
use crate::gui::force_unwrapped_field::ForceUnwrappedField;
use gio::ListModel;
use glib::Type;
use libadwaita::subclass::prelude::*;
use libadwaita::{gio, glib};

#[derive(Default)]
pub struct DatabaseListModel {
	backend: Backend,
	database_view: ForceUnwrappedField<Box<dyn DynamicDatabaseView>>,
}

#[glib::object_subclass]
impl ObjectSubclass for DatabaseListModel {
	const NAME: &'static str = "BabyNameTournamentDatabaseListModel";
	type Type = super::DatabaseListModel;
	type ParentType = glib::Object;
	type Interfaces = (ListModel,);
}

impl ObjectImpl for DatabaseListModel {}
impl ListModelImpl for DatabaseListModel {
	fn item_type(&self) -> Type {
		self.database_view.gobject_model()
	}

	fn n_items(&self) -> u32 {
		self.backend
			.block_on_future(self.database_view.count(self.backend.database_pool()))
	}

	fn item(&self, position: u32) -> Option<glib::Object> {
		self.backend
			.block_on_future(
				self.database_view
					.read_at_offset(self.backend.database_pool(), position),
			)
			.map(|row| self.database_view.convert(row))
			.ok()
	}
}

impl DatabaseListModel {
	pub fn initialize(&self, backend: Backend, database_view: impl DatabaseView) {
		self.backend.initialize(backend);
		self.database_view.initialize(Box::new(database_view));
	}
}
