use crate::csv_parser::Gender;
use crate::database;
use crate::database::views::NameWithPreferences;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list::DatabaseView;
use crate::gui::name_list::{NameList, NameListInput, NameListOutput};
use gtk::{Adjustment, Align, Label, Orientation};
use libadwaita::prelude::*;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent};

pub struct NameDetailView {
	name: Name,
	similar_name_list_controller: Controller<NameList<SimilarNameListView>>,
	filter: SimilarNameListViewFilter,
}

#[relm4::component(pub)]
impl SimpleComponent for NameDetailView {
	type Input = NameDetailViewInput;
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

			gtk::Box {
				set_orientation: Orientation::Horizontal,
				set_spacing: 12,
				set_halign: Align::Center,
				Label {
					set_label: "Levenshtein Threshold:",
				},
				gtk::SpinButton {
					#[watch]
					set_value: model.filter.threshold,
					set_adjustment: &Adjustment::new(3.0, 0.0, 20.0, 1.0, 1.0, 1.0),
					connect_value_changed[sender] => move |button| {
						let _ = sender.input_sender().send(NameDetailViewInput::UpdateThreshold(button.value()));
					},
				},
			},

			#[local]
			similar_name_list -> gtk::Box {}
		}
	}

	fn init((backend, name): Self::Init, _root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let filter = SimilarNameListViewFilter {
			name: name.name.clone(),
			gender: name.gender,
			threshold: 3.0,
		};

		let similar_name_list_controller = NameList::<SimilarNameListView>::builder()
			.launch((filter.clone(), backend))
			.forward(sender.input_sender(), |output| match output {
				NameListOutput::NamePreferenceUpdated(name_with_preferences) => {
					NameDetailViewInput::UpdateNamePreferences(name_with_preferences)
				}
				NameListOutput::NameSelected(name) => NameDetailViewInput::SetName(name),
			});

		let similar_name_list = similar_name_list_controller.widget().clone();

		let model = NameDetailView {
			name,
			filter,
			similar_name_list_controller,
		};

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NameDetailViewInput::*;
		match message {
			SetName(name) => {
				self.name = name;
				self.filter.name = self.name.name.clone();
				self.filter.threshold = 3.0;
			}
			SetGender(gender) => {
				self.filter.gender = gender;
			}
			UpdateThreshold(threshold) => {
				self.filter.threshold = threshold;
			}
			UpdateNamePreferences(name_with_preferences) => {
				let _ = sender
					.output_sender()
					.send(NameDetailViewOutput::NamePreferenceSet(name_with_preferences));
				return; // don't update the filter
			}
		};

		let _ = self
			.similar_name_list_controller
			.sender()
			.send(NameListInput::UpdateFilter(self.filter.clone()));
	}
}

#[derive(Debug)]
pub enum NameDetailViewInput {
	SetName(Name),
	SetGender(Gender),
	UpdateNamePreferences(NameWithPreferences),
	UpdateThreshold(f64),
}

#[derive(Debug)]
pub enum NameDetailViewOutput {
	NamePreferenceSet(NameWithPreferences),
}

#[derive(Clone, Default)]
struct SimilarNameListView;

#[derive(Clone, Debug)]
struct SimilarNameListViewFilter {
	name: String,
	gender: Gender,
	threshold: f64,
}

impl DatabaseView for SimilarNameListView {
	type Model = NameWithPreferences;
	type Filter = SimilarNameListViewFilter;

	fn read_at_offset(
		&self,
		backend: &Backend,
		SimilarNameListViewFilter {
			name,
			gender,
			threshold,
		}: &Self::Filter,
		offset: u32,
	) -> anyhow::Result<Self::Model> {
		Ok(backend.block_on_future(database::views::read_similar_at_offset(
			name,
			*gender,
			*threshold,
			offset.into(),
			backend.database_pool(),
		))?)
	}

	fn count(
		&self,
		backend: &Backend,
		SimilarNameListViewFilter {
			name,
			gender,
			threshold,
		}: &Self::Filter,
	) -> u32 {
		backend
			.block_on_future(database::views::count_similar(
				name,
				*gender,
				*threshold,
				backend.database_pool(),
			))
			.unwrap_or_default()
			.try_into()
			.expect("Number out of range")
	}
}
