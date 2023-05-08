use crate::database::Name;
use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct NameListRow {
	name: Option<Name>,
}

#[relm4::component(pub)]
impl SimpleComponent for NameListRow {
	type Input = Name;
	type Output = std::convert::Infallible;
	type Init = ();

	view! {
		gtk::Box {
			set_homogeneous: true,

			#[name(name_label)]
			gtk::Label {
				#[watch]
				set_label: model.name.as_ref().map(|Name {name, ..}| name.as_str()).unwrap_or("<unknown>"),
			},

			#[name(gender_label)]
			gtk::Label {
				#[watch]
				set_label: model.name.as_ref().map(|Name {gender, ..}| gender.as_ref()).unwrap_or("<unknown>"),
			}
		}
	}

	fn init((): Self::Init, _root: &Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let model = NameListRow { name: None };

		let widgets = view_output!();

		ComponentParts { widgets, model }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.name = Some(message);
	}
}
