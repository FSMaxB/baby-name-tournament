use crate::gui::name_list::NameList;
use glib::Object;
use libadwaita::subclass::prelude::*;
use libadwaita::Application;
use libadwaita::{gio, glib, gtk};
use std::ops::Deref;

mod implementation;

glib::wrapper! {
	pub struct Window(ObjectSubclass<implementation::Window>)
		@extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, libadwaita::ApplicationWindow,
		@implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
	pub fn new(application: &Application) -> Self {
		Object::builder().property("application", application).build()
	}

	pub fn name_list(&self) -> &NameList {
		self.imp().name_list.deref()
	}
}
