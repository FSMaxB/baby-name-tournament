use libadwaita::{glib, gtk};

mod implementation;

glib::wrapper! {
	pub struct NameList(ObjectSubclass<implementation::NameList>)
		@extends gtk::Widget, gtk::Box,
		@implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
