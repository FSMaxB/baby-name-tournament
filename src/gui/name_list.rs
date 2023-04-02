use crate::gui::backend::Backend;
use libadwaita::subclass::prelude::*;
use libadwaita::{glib, gtk};

mod implementation;

glib::wrapper! {
	pub struct NameList(ObjectSubclass<implementation::NameList>)
		@extends gtk::Widget, gtk::Box,
		@implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl NameList {
	pub fn initialize(&self, backend: Backend) {
		self.imp().backend.initialize(backend);
		self.imp().schedule_update_of_all_names();
	}
}
