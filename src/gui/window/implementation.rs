use glib::subclass::InitializingObject;
use gtk::{CompositeTemplate, TemplateChild};
use libadwaita::subclass::prelude::*;
use libadwaita::{glib, gtk, ApplicationWindow};
use crate::gui::name_list::NameList;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../../resources/ui/window.ui")]
pub struct Window {
	#[template_child]
	pub name_list: TemplateChild<NameList>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
	const NAME: &'static str = "BabyNameTournamentWindow";
	type Type = super::Window;
	type ParentType = ApplicationWindow;

	fn class_init(class: &mut Self::Class) {
		class.bind_template();
	}

	fn instance_init(object: &InitializingObject<Self>) {
		object.init_template();
	}
}

impl ObjectImpl for Window {}
impl WidgetImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}
impl AdwApplicationWindowImpl for Window {}
