use crate::gui::database_list::database_list_manager::{DatabaseListManager, DynamicListManager};
use crate::gui::database_list::DatabaseView;
use crate::gui::force_unwrapped_field::ForceUnwrappedField;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gio::{prelude::*, ListModel};
use glib::{clone, BoxedAnyObject, Type};
use relm4::adw;

#[derive(Default)]
pub struct DatabaseListModel {
	database_view: ForceUnwrappedField<Box<dyn DynamicListManager>>,
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
		BoxedAnyObject::static_type()
	}

	fn n_items(&self) -> u32 {
		self.database_view.count()
	}

	fn item(&self, position: u32) -> Option<glib::Object> {
		self.database_view.read_at_offset(position).map(Cast::upcast).ok()
	}
}

impl DatabaseListModel {
	pub fn initialize(&self, database_list_manager: DatabaseListManager<impl DatabaseView>) {
		let this = self.obj();
		database_list_manager.register_items_changed_callback(Box::new(clone!(
			#[weak]
			this,
			move |position, removed, added| {
				this.items_changed(position, removed, added);
			}
		)));

		self.database_view.initialize(database_list_manager.erase());
	}
}
