use crate::csv_parser::Gender;
use crate::database::views::NameWithPreferences;
use crate::gui::backend::Backend;
use crate::gui::gender_dropdown::GenderDropdown;
use crate::gui::main_view::preference_filter::{PreferenceFilter, PreferenceFilterComponent, PreferenceFilterOutput};
use crate::gui::name_list::{NameList, NameListInput, NameListOutput, NameListView, NameListViewFilter};
use gtk::{prelude::*, Orientation};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent};

mod preference_filter;

pub struct MainView {
	name_list_controller: Controller<NameList<NameListView>>,
	_gender_filter_controller: Controller<GenderDropdown>,
	_name_preference_controller: Controller<PreferenceFilterComponent>,
	filter: NameListViewFilter,
}

#[derive(Debug)]
pub enum MainViewInput {
	GenderSelected(Gender),
	NamePreferenceUpdated(NameWithPreferences),
	UpdateNamePreferenceFilter(PreferenceFilter),
	UpdateSearchTerm(String),
	RefreshRow { name: String },
}

#[derive(Debug)]
pub enum MainViewOutput {
	NamePreferenceUpdated(NameWithPreferences),
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

			#[local]
			name_preference_view -> gtk::Box {},

			#[local]
			name_list -> gtk::Box {},
		}
	}

	fn init(backend: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let name_list_controller = NameList::builder()
			.launch((NameListViewFilter::default(), backend))
			.forward(sender.input_sender(), |output| match output {
				NameListOutput::NamePreferenceUpdated(name_with_preferences) => {
					MainViewInput::NamePreferenceUpdated(name_with_preferences)
				}
			});
		let name_list = name_list_controller.widget().clone();

		let gender_dropdown_controller = GenderDropdown::builder()
			.launch(())
			.forward(sender.input_sender(), MainViewInput::GenderSelected);
		let gender_dropdown = gender_dropdown_controller.widget().clone();

		let filter = NameListViewFilter::default();
		let name_preference_controller = PreferenceFilterComponent::builder()
			.launch(PreferenceFilter {
				show_favorite: filter.show_favorite,
				show_nogo: filter.show_nogo,
				show_undecided: filter.show_undecided,
			})
			.forward(sender.input_sender(), |message| match message {
				PreferenceFilterOutput::UpdateFilter(filter) => MainViewInput::UpdateNamePreferenceFilter(filter),
			});
		let name_preference_view = name_preference_controller.widget().clone();

		let model = Self {
			name_list_controller,
			_gender_filter_controller: gender_dropdown_controller,
			_name_preference_controller: name_preference_controller,
			filter,
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
			}
			NamePreferenceUpdated(name_with_preferences) => {
				let _ = sender.output(MainViewOutput::NamePreferenceUpdated(name_with_preferences));
			}
			UpdateNamePreferenceFilter(PreferenceFilter {
				show_favorite,
				show_nogo,
				show_undecided,
			}) => {
				self.filter.show_favorite = show_favorite;
				self.filter.show_nogo = show_nogo;
				self.filter.show_undecided = show_undecided;

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
			RefreshRow { name } => {
				let _ = self
					.name_list_controller
					.sender()
					.send(NameListInput::RefreshRow { name });
			}
		}
	}
}
