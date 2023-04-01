use crate::database::Name;
use glib::Object;
use libadwaita::glib;

mod implementation;

glib::wrapper! {
	pub struct NameModel(ObjectSubclass<implementation::NameModel>);
}

impl From<Name> for NameModel {
	fn from(Name { name, gender }: Name) -> Self {
		Object::builder()
			.property("name", name)
			.property("gender", gender.to_string())
			.build()
	}
}
