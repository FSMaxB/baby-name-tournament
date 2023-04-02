use crate::gui::backend::Backend;
use crate::gui::name_model::NameModel;
use gio::ListStore;
use glib::subclass::InitializingObject;
use gtk::{
	prelude::*, ColumnView, ColumnViewColumn, CompositeTemplate, Label, SignalListItemFactory, SingleSelection,
	TemplateChild, Widget,
};
use libadwaita::glib::{clone, MainContext};
use libadwaita::subclass::prelude::*;
use libadwaita::{gio, glib, gtk};
use std::ops::Deref;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../../resources/ui/name_list.ui")]
pub struct NameList {
	#[template_child]
	pub name_list: TemplateChild<ColumnView>,
	pub list_store: ListStore,
	pub selection_model: SingleSelection,
	// TODO: Use custom Model instead
	pub backend: Backend,
}

#[glib::object_subclass]
impl ObjectSubclass for NameList {
	const NAME: &'static str = "BabyNameTournamentNameList";
	type Type = super::NameList;
	type ParentType = gtk::Box;

	fn class_init(class: &mut Self::Class) {
		class.bind_template();
	}

	fn instance_init(object: &InitializingObject<Self>) {
		object.init_template();
	}
}

impl ObjectImpl for NameList {
	fn constructed(&self) {
		self.parent_constructed();

		let name_list = self.name_list.deref();
		self.selection_model.set_model(Some(&self.list_store));

		let name_factory = SignalListItemFactory::new();
		let gender_factory = SignalListItemFactory::new();

		name_factory.connect_setup(|_, list_item| {
			let name_label = Label::new(Some("<empty name>"));
			list_item.set_child(Some(&name_label));
			list_item
				.property_expression("item")
				.chain_property::<NameModel>("name")
				.bind(&name_label, "label", Widget::NONE);
		});
		gender_factory.connect_setup(|_, list_item| {
			let gender_label = Label::new(Some("<empty gender>"));
			list_item.set_child(Some(&gender_label));
			list_item
				.property_expression("item")
				.chain_property::<NameModel>("gender")
				.bind(&gender_label, "label", Widget::NONE);
		});

		let name_column = ColumnViewColumn::builder()
			.title("Name")
			.resizable(true)
			.factory(&name_factory)
			.build();
		let gender_column = ColumnViewColumn::builder()
			.title("Gender")
			.resizable(true)
			.factory(&gender_factory)
			.build();

		name_list.set_model(Some(&self.selection_model));
		name_list.append_column(&name_column);
		name_list.append_column(&gender_column);
	}
}
impl WidgetImpl for NameList {}
impl BoxImpl for NameList {}

impl NameList {
	pub fn schedule_update_of_all_names(&self) {
		let backend = self.backend.clone();
		let list_store = &self.list_store;
		MainContext::default().spawn_local(clone!(@weak list_store => async move {
			let names = backend.list_all_names().await;
			list_store.remove_all();
			for name in names {
				list_store.append(&NameModel::from(name));
			}
		}));
	}
}
