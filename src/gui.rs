use crate::csv_parser::Gender;
use crate::gui::name_detail_view::{NameDetailViewInput, NameDetailViewOutput};
use crate::gui::runtime_thread::RuntimeThread;
use gtk::{Orientation, StackTransitionType};
use libadwaita::prelude::*;
use libadwaita::HeaderBar;
use relm4::{
	gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, SimpleComponent,
};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

const APPLICATION_ID: &str = "de.maxbruckner.baby-name-tournament";

mod backend;
mod database_list;
mod force_unwrapped_field;
mod gender_dropdown;
mod main_view;
mod name_detail_view;
mod name_list;
mod name_preference;
mod runtime_thread;

use crate::database;
use crate::database::views::NameWithPreferences;
use crate::database::Name;
use crate::gui::main_view::{MainView, MainViewInput, MainViewOutput};
use crate::gui::name_detail_view::NameDetailView;
use backend::Backend;

pub fn start(runtime: Runtime, database_pool: SqlitePool) -> anyhow::Result<()> {
	let runtime_thread = RuntimeThread::start(runtime);
	let handle = runtime_thread.handle().clone();

	RelmApp::new(APPLICATION_ID).run::<Application>(Backend::new(database_pool.clone(), handle.clone()));

	handle.block_on(database_pool.close());

	runtime_thread.shut_down()
}

struct Application {
	main_view_controller: Controller<MainView>,
	name_detail_view_controller: Controller<NameDetailView>,
	stack: gtk::Stack,
	back_button: gtk::Button,
	backend: Backend,
}

#[derive(Debug)]
enum ApplicationMessage {
	GenderSelected(Gender),
	NameSelected(Name),
	NamePreferenceUpdated(NameWithPreferences),
	BackToList,
}

#[relm4::component]
impl SimpleComponent for Application {
	type Init = Backend;
	type Input = ApplicationMessage;
	type Output = ();

	view! {
		libadwaita::ApplicationWindow {
			set_title: Some("Baby Name Tournament"),
			set_default_size: (480, 640),

			gtk::Box {
				set_orientation: Orientation::Vertical,

				HeaderBar {
					pack_start: &back_button,
				},

				#[local]
				stack -> gtk::Stack {
					set_transition_type: StackTransitionType::SlideLeftRight,

					#[local]
					main_view -> gtk::Box {},

					#[local]
					name_detail_view -> gtk::Box {},
				},
			}
		}
	}

	fn init(backend: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let main_view_controller =
			MainView::builder()
				.launch(backend.clone())
				.forward(sender.input_sender(), |message| match message {
					MainViewOutput::NamePreferenceUpdated(name_with_preference) => {
						ApplicationMessage::NamePreferenceUpdated(name_with_preference)
					}
					MainViewOutput::NameSelected(name) => ApplicationMessage::NameSelected(name),
					MainViewOutput::GenderSelected(gender) => ApplicationMessage::GenderSelected(gender),
				});
		let main_view = main_view_controller.widget().clone();

		let name_detail_view_controller = NameDetailView::builder()
			.launch((
				backend.clone(),
				Name {
					name: "<none>".to_owned(),
					gender: Gender::Both,
				},
			))
			.forward(sender.input_sender(), |message| match message {
				NameDetailViewOutput::NamePreferenceSet(name_with_preferences) => {
					ApplicationMessage::NamePreferenceUpdated(name_with_preferences)
				}
			});
		let name_detail_view = name_detail_view_controller.widget().clone();

		let stack = gtk::Stack::new();

		let back_button = gtk::Button::builder()
			.name("Back")
			.icon_name("go-previous-symbolic")
			.visible(false)
			.build();

		back_button.connect_clicked({
			let input_sender = sender.input_sender().clone();
			move |_| {
				let _ = input_sender.send(ApplicationMessage::BackToList);
			}
		});

		let model = Self {
			main_view_controller,
			name_detail_view_controller,
			stack: stack.clone(),
			back_button: back_button.clone(),
			backend,
		};

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		use ApplicationMessage::*;
		match message {
			GenderSelected(gender) => {
				let _ = self
					.name_detail_view_controller
					.sender()
					.send(NameDetailViewInput::SetGender(gender));
			}
			NameSelected(name) => {
				let _ = self
					.name_detail_view_controller
					.sender()
					.send(NameDetailViewInput::SetName(name));
				// TODO: Make this better than based on position
				self.stack.pages().select_item(1, true);
				self.back_button.set_visible(true);
			}
			NamePreferenceUpdated(NameWithPreferences {
				name,
				mother_preference,
				father_preference,
				..
			}) => {
				self.backend
					.block_on_future(database::upsert_name_preference(
						&name,
						mother_preference,
						father_preference,
						self.backend.database_pool(),
					))
					.expect("Failed to update name preference");
				let _ = self
					.name_detail_view_controller
					.sender()
					.send(NameDetailViewInput::RefreshRow { name: name.clone() });
				let _ = self
					.main_view_controller
					.sender()
					.send(MainViewInput::RefreshRow { name: name });
			}
			BackToList => {
				// TODO: Make this better than based on position
				self.stack.pages().select_item(0, true);
				self.back_button.set_visible(false);
				let _ = self.main_view_controller.sender().send(MainViewInput::RefreshAll);
			}
		}
	}
}
