use crate::csv_parser::Gender;
use crate::database::views::NameWithPreferences;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::gender_dropdown::GenderDropdown;
use crate::gui::name_list::{NameList, NameListInput, NameListOutput, NameListView, NameListViewFilter};
use gtk::{prelude::*, Align, Orientation};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent};

pub struct MainView {
	name_list_controller: Controller<NameList<NameListView>>,
	show_favorite_checkbox: gtk::CheckButton,
	show_nogo_checkbox: gtk::CheckButton,
	show_neutral_checkbox: gtk::CheckButton,
	filter: NameListViewFilter,
}

#[derive(Debug)]
pub enum MainViewInput {
	GenderSelected(Gender),
	NameSelected(Name),
	NamePreferenceUpdated(NameWithPreferences),
	UpdateNamePreferenceFilter,
	UpdateSearchTerm(String),
	Refresh,
}

#[derive(Debug)]
pub enum MainViewOutput {
	NamePreferenceUpdated(NameWithPreferences),
	NameSelected(Name),
	GenderSelected(Gender),
}

#[relm4::component(pub)]
impl SimpleComponent for MainView {
	type Input = MainViewInput;
	type Output = MainViewOutput;
	type Init = Backend;

	view! {
		gtk::Box {
			set_orientation: Orientation::Vertical,

			gtk::Box {
				set_orientation: Orientation::Horizontal,
				set_homogeneous: true,

				gtk::SearchEntry {
					set_placeholder_text: Some("Search ..."),
					connect_search_changed[sender] => move |search_field| {
						sender.input(MainViewInput::UpdateSearchTerm(search_field.text().as_str().to_owned()));
					}
				},
			},

			#[local]
			gender_dropdown -> gtk::DropDown {},

			gtk::Box {
				set_orientation: Orientation::Horizontal,
				set_halign: Align::Center,
				set_spacing: 12,

				#[local]
				show_favorite_checkbox -> gtk::CheckButton {
					set_active: model.filter.show_favorite,
					connect_toggled[sender] => move |_| {
						sender.input(MainViewInput::UpdateNamePreferenceFilter);
					}
				},
				gtk::Image {
					set_from_icon_name: Some("emblem-favorite-symbolic"),
				},

				#[local]
				show_nogo_checkbox -> gtk::CheckButton {
					set_active: model.filter.show_nogo,
					connect_toggled[sender] => move |_| {
						sender.input(MainViewInput::UpdateNamePreferenceFilter);
					}
				},
				gtk::Image {
					set_from_icon_name: Some("action-unavailable-symbolic"),
				},

				#[local]
				show_neutral_checkbox -> gtk::CheckButton {
					set_label: Some("Neutral"),
					set_active: model.filter.show_neutral,
					connect_toggled[sender] => move |_| {
						sender.input(MainViewInput::UpdateNamePreferenceFilter);
					}
				},
			},

			#[local]
			name_list -> gtk::Box {},
		}
	}

	fn init(backend: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let name_list_controller = NameList::builder()
			.launch((NameListViewFilter::default(), backend))
			.forward(sender.input_sender(), |output| match output {
				NameListOutput::NameSelected(name) => MainViewInput::NameSelected(name),
				NameListOutput::NamePreferenceUpdated(name_with_preferences) => {
					MainViewInput::NamePreferenceUpdated(name_with_preferences)
				}
			});
		let name_list = name_list_controller.widget().clone();

		let gender_dropdown_controller = GenderDropdown::builder()
			.launch(())
			.forward(sender.input_sender(), MainViewInput::GenderSelected);
		let gender_dropdown = gender_dropdown_controller.widget().clone();

		let show_favorite_checkbox = gtk::CheckButton::new();
		let show_nogo_checkbox = gtk::CheckButton::new();
		let show_neutral_checkbox = gtk::CheckButton::new();
		let model = Self {
			name_list_controller,
			show_favorite_checkbox: show_favorite_checkbox.clone(),
			show_nogo_checkbox: show_nogo_checkbox.clone(),
			show_neutral_checkbox: show_neutral_checkbox.clone(),
			filter: Default::default(),
		};

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use MainViewInput::*;
		match message {
			GenderSelected(gender) => {
				self.filter.gender = gender;
				let _ = self
					.name_list_controller
					.sender()
					.send(NameListInput::UpdateFilter(self.filter.clone()));
				let _ = sender.output(MainViewOutput::GenderSelected(gender));
			}
			NameSelected(name) => {
				let _ = sender.output(MainViewOutput::NameSelected(name));
			}
			NamePreferenceUpdated(name_with_preferences) => {
				let _ = sender.output(MainViewOutput::NamePreferenceUpdated(name_with_preferences));
			}
			UpdateNamePreferenceFilter => {
				self.filter.show_favorite = self.show_favorite_checkbox.is_active();
				self.filter.show_nogo = self.show_nogo_checkbox.is_active();
				self.filter.show_neutral = self.show_neutral_checkbox.is_active();

				let _ = self
					.name_list_controller
					.sender()
					.send(NameListInput::UpdateFilter(self.filter.clone()));
			}
			UpdateSearchTerm(search_term) => {
				self.filter.name_contains = if search_term.trim().is_empty() {
					None
				} else {
					Some(search_term)
				};

				let _ = self
					.name_list_controller
					.sender()
					.send(NameListInput::UpdateFilter(self.filter.clone()));
			}
			Refresh => {
				let _ = self.name_list_controller.sender().send(NameListInput::Refresh);
			}
		}
	}
}
