use crate::database::views::NameWithPreference;
use crate::database::{Name, NamePreference};
use crate::gtk::name_preference::{NamePreferenceInput, NamePreferenceView};
use gtk::prelude::*;
use relm4::{
	Component, ComponentController, ComponentParts, ComponentSender, Controller, Sender, SimpleComponent, gtk,
};

pub struct NameListRow {
	name: Name,
	preference: Option<NamePreference>,
	preference_controller: Controller<NamePreferenceView>,
}

#[relm4::component(pub)]
impl SimpleComponent for NameListRow {
	type Input = NameListRowInput;
	type Output = NameListRowOutput;
	type Init = NameListRowInit;

	view! {
		gtk::Box {
			set_homogeneous: true,

			#[name(name_label)]
			gtk::Label {
				set_use_markup: true,
				#[watch]
				set_label: &format!(r"<big><b>{}</b></big>", model.name.name),
			},

			#[name(gender_label)]
			gtk::Label {
				#[watch]
				set_label: model.name.gender.as_ref(),
			},

			#[local]
			preference_widget -> gtk::Box {},

			gtk::Box {
				gtk::Button {
					set_icon_name: "edit-undo-symbolic",
					set_vexpand: false,
					set_hexpand: false,

					connect_clicked[sender] => move |_| {
						sender.input(NameListRowInput::UpdatePreference(None));
					}
				},
			}

		}
	}

	fn init(
		NameListRowInit { name, preference }: Self::Init,
		root: Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let preference_controller = NamePreferenceView::builder()
			.launch(("Preference", None))
			.forward(sender.input_sender(), NameListRowInput::UpdatePreference);
		let preference_widget = preference_controller.widget().clone();

		let model = NameListRow {
			name,
			preference,
			preference_controller,
		};

		let widgets = view_output!();

		ComponentParts { widgets, model }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NameListRowInput::*;
		match message {
			SetName(NameWithPreference {
				name,
				gender,
				preference,
			}) => {
				self.name = Name { name, gender };
				self.preference = preference;
				let _ = self
					.preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(preference));
			}
			UpdatePreference(preference) => {
				self.preference = preference;
				let _ = self
					.preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(preference));

				self.send_preference_output(sender.output_sender());
			}
		}

		let _ = self
			.preference_controller
			.sender()
			.send(NamePreferenceInput::SetPreference(self.preference));
	}
}

impl NameListRow {
	fn send_preference_output(&self, sender: &Sender<NameListRowOutput>) {
		let _ = sender.send(NameListRowOutput::NamePreferenceSet(NameWithPreference {
			name: self.name.name.clone(),
			gender: self.name.gender,
			preference: self.preference,
		}));
	}
}

#[derive(Debug)]
pub struct NameListRowInit {
	pub name: Name,
	pub preference: Option<NamePreference>,
}

#[derive(Debug)]
pub enum NameListRowInput {
	SetName(NameWithPreference),
	UpdatePreference(Option<NamePreference>),
}

#[derive(Debug)]
pub enum NameListRowOutput {
	NamePreferenceSet(NameWithPreference),
}
