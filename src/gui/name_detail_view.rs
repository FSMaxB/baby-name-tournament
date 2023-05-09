use crate::database::Name;
use crate::gui::backend::Backend;
use gtk::{Align, Label, Orientation};
use libadwaita::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct NameDetailView {
	name: Name,
	_backend: Backend,
}

#[relm4::component(pub)]
impl SimpleComponent for NameDetailView {
	type Input = Name;
	type Output = std::convert::Infallible;
	type Init = (Backend, Name);

	view! {
		gtk::Box {
			set_orientation: Orientation::Vertical,

			gtk::Box {
				set_orientation: Orientation::Horizontal,
				set_spacing: 12,
				set_halign: Align::Center,
				Label {
					set_label: "Name:",
				},
				Label {
					#[watch]
					set_label: &model.name.name,
				},
			},
		}
	}

	fn init((_backend, name): Self::Init, _root: &Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let model = NameDetailView { name, _backend };

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.name = message;
	}
}
