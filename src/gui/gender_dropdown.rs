use crate::csv_parser::Gender;
use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use strum::VariantNames;

pub struct GenderDropdown;

#[relm4::component(pub)]
impl SimpleComponent for GenderDropdown {
	type Input = ();
	type Output = Gender;
	type Init = ();

	view! {
		gtk::DropDown {
			set_model: Some(&gtk::StringList::new(Gender::VARIANTS)),
			connect_selected_item_notify[sender] => move |dropdown| {
				sender.output(selected_gender(dropdown)).expect("Failed to send output");
			}
		}
	}

	fn init((): Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let widgets = view_output!();

		sender.output(selected_gender(root)).expect("Failed to send gender");

		ComponentParts { model: Self, widgets }
	}
}

fn selected_gender(dropdown: &gtk::DropDown) -> Gender {
	let item = dropdown
		.selected_item()
		.expect("No item was selected")
		.downcast::<gtk::StringObject>()
		.expect("Wasn't a GtkStringObject")
		.string();
	item.as_str().parse().expect("Invalid gender string")
}
