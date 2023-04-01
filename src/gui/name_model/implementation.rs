use glib::prelude::*;
use glib::{ParamSpec, Properties, Value};
use libadwaita::glib;
use libadwaita::subclass::prelude::*;
use std::cell::RefCell;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::NameModel)]
pub struct NameModel {
	#[property(get, set)]
	name: RefCell<String>,
	// TODO: verify correct value
	#[property(get, set)]
	gender: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for NameModel {
	const NAME: &'static str = "BabyNameTournamentNameModel";
	type Type = super::NameModel;
}

impl ObjectImpl for NameModel {
	fn properties() -> &'static [ParamSpec] {
		Self::derived_properties()
	}

	fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
		self.derived_set_property(id, value, pspec);
	}

	fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
		self.derived_property(id, pspec)
	}
}
