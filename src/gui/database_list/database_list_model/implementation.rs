use crate::gui::backend::Backend;
use crate::gui::database_list::database_list_model::DynamicDatabaseView;
use crate::gui::database_list::DatabaseView;
use crate::gui::force_unwrapped_field::ForceUnwrappedField;
use gio::{prelude::*, ListModel};
use glib::Type;
use libadwaita::glib::clone;
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
		self.database_view.count(&self.backend)
	}

	fn item(&self, position: u32) -> Option<glib::Object> {
		self.database_view
			.read_at_offset(&self.backend, position)
			.map(|row| self.database_view.convert(row))
			.ok()
	}
}

impl DatabaseListModel {
	pub fn initialize(&self, backend: Backend, database_view: impl DatabaseView) {
		self.backend.initialize(backend);
		self.database_view.initialize(Box::new(database_view));
		let this = self.obj();
		self.database_view.register_items_changed_callback(Box::new(
			clone!(@weak this => move |previous_count, count| {
				this.items_changed(0, previous_count, count);
			}),
		));
	}
}
