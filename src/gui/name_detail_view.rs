use crate::database::Name;
use crate::gui::backend::Backend;
use gtk::{Align, Button, Label, Orientation};
use libadwaita::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct NameDetailView {
	name: Name,
	_backend: Backend,
}

#[relm4::component(pub)]
impl SimpleComponent for NameDetailView {
	type Input = Name;
	type Output = NameDetailViewOutput;
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
			Button {
				set_label: "Back",

				connect_clicked[sender] => move |_| {
					let _ = sender.output(NameDetailViewOutput::Back);
				},
			}
		}
	}

	fn init((_backend, name): Self::Init, _root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let model = NameDetailView { name, _backend };

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.name = message;
	}
}

#[derive(Debug)]
pub enum NameDetailViewOutput {
	Back,
}
