use crate::gui::backend::Backend;
use crate::gui::database_list_model::{DatabaseListModel, DatabaseView};
use crate::gui::name_model::NameModel;
use glib::subclass::InitializingObject;
use gtk::{
	prelude::*, ColumnView, ColumnViewColumn, CompositeTemplate, Label, SignalListItemFactory, SingleSelection,
	TemplateChild, Widget,
};
use libadwaita::subclass::prelude::*;
use libadwaita::{glib, gtk};
use std::ops::Deref;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../../resources/ui/name_list.ui")]
pub struct NameList {
	#[template_child]
	pub name_list: TemplateChild<ColumnView>,
	pub database_list_model: DatabaseListModel,
	pub selection_model: SingleSelection,
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
	pub fn initialize(&self, backend: Backend, database_view: impl DatabaseView) {
		self.database_list_model.initialize(backend, database_view);

		self.selection_model.set_model(Some(&self.database_list_model));
	}
}
