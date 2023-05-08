use crate::database::Name;
use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct NameListRow {
	name: Name,
}

#[relm4::component(pub)]
impl SimpleComponent for NameListRow {
	type Input = Name;
	type Output = std::convert::Infallible;
	type Init = Name;

	view! {
		gtk::Box {
			set_homogeneous: true,

			#[name(name_label)]
			gtk::Label {
				#[watch]
				set_label: model.name.name.as_str(),
			},

			#[name(gender_label)]
			gtk::Label {
				#[watch]
				set_label: model.name.gender.as_ref(),
			}
		}
	}

	fn init(name: Self::Init, _root: &Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let model = NameListRow { name };

		let widgets = view_output!();

		ComponentParts { widgets, model }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.name = message;
	}
}
