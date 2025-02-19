use crate::database::views::NameWithPreferences;
use crate::database::{Name, NamePreference};
use crate::gui::name_preference::{NamePreferenceInput, NamePreferenceView};
use gtk::prelude::*;
use relm4::{
	gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, Sender, SimpleComponent,
};

pub struct NameListRow {
	name: Name,
	mother_preference: Option<NamePreference>,
	father_preference: Option<NamePreference>,
	shared_preference_controller: Controller<NamePreferenceView>,
	mother_preference_controller: Controller<NamePreferenceView>,
	father_preference_controller: Controller<NamePreferenceView>,
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
			shared_preference_widget -> gtk::Box {},

			#[local]
			mother_preference_widget -> gtk::Box {},
			#[local]
			father_preference_widget -> gtk::Box {},

			gtk::Box {
				gtk::Button {
					set_icon_name: "edit-undo-symbolic",
					set_vexpand: false,
					set_hexpand: false,

					connect_clicked[sender] => move |_| {
						sender.input(NameListRowInput::UpdateSharedPreference(None));
					}
				},
			}

		}
	}

	fn init(
		NameListRowInit {
			name,
			mother_preference,
			father_preference,
		}: Self::Init,
		root: Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let shared_preference_controller = NamePreferenceView::builder()
			.launch(("Shared Preference", None))
			.forward(sender.input_sender(), NameListRowInput::UpdateSharedPreference);
		let shared_preference_widget = shared_preference_controller.widget().clone();

		let mother_preference_controller = NamePreferenceView::builder()
			.launch(("Mother", mother_preference))
			.forward(sender.input_sender(), NameListRowInput::UpdateMotherPreference);
		let mother_preference_widget = mother_preference_controller.widget().clone();

		let father_preference_controller = NamePreferenceView::builder()
			.launch(("Father", father_preference))
			.forward(sender.input_sender(), NameListRowInput::UpdateFatherPreference);
		let father_preference_widget = father_preference_controller.widget().clone();

		let model = NameListRow {
			name,
			mother_preference,
			father_preference,
			shared_preference_controller,
			mother_preference_controller,
			father_preference_controller,
		};

		let widgets = view_output!();

		ComponentParts { widgets, model }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NameListRowInput::*;
		match message {
			SetName(NameWithPreferences {
				name,
				gender,
				mother_preference,
				father_preference,
			}) => {
				self.name = Name { name, gender };
				self.mother_preference = mother_preference;
				self.father_preference = father_preference;
				let _ = self
					.mother_preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(mother_preference));
				let _ = self
					.father_preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(father_preference));
			}
			UpdateSharedPreference(preference) => {
				self.mother_preference = preference;
				self.father_preference = preference;
				let _ = self
					.mother_preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(preference));
				let _ = self
					.father_preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(preference));

				self.send_preference_output(sender.output_sender());
			}
			UpdateMotherPreference(preference) => {
				self.mother_preference = preference;
				let _ = self
					.mother_preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(preference));
				self.send_preference_output(sender.output_sender());
			}
			UpdateFatherPreference(preference) => {
				self.father_preference = preference;
				let _ = self
					.father_preference_controller
					.sender()
					.send(NamePreferenceInput::SetPreference(preference));
				self.send_preference_output(sender.output_sender());
			}
		}

		let shared_preference = if self.mother_preference == self.father_preference {
			self.mother_preference
		} else {
			None
		};
		let _ = self
			.shared_preference_controller
			.sender()
			.send(NamePreferenceInput::SetPreference(shared_preference));
	}
}

impl NameListRow {
	fn send_preference_output(&self, sender: &Sender<NameListRowOutput>) {
		let _ = sender.send(NameListRowOutput::NamePreferenceSet(NameWithPreferences {
			name: self.name.name.clone(),
			gender: self.name.gender,
			mother_preference: self.mother_preference,
			father_preference: self.father_preference,
		}));
	}
}

#[derive(Debug)]
pub struct NameListRowInit {
	pub name: Name,
	pub mother_preference: Option<NamePreference>,
	pub father_preference: Option<NamePreference>,
}

#[derive(Debug)]
pub enum NameListRowInput {
	SetName(NameWithPreferences),
	UpdateSharedPreference(Option<NamePreference>),
	UpdateMotherPreference(Option<NamePreference>),
	UpdateFatherPreference(Option<NamePreference>),
}

#[derive(Debug)]
pub enum NameListRowOutput {
	NamePreferenceSet(NameWithPreferences),
}
