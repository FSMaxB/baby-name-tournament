use crate::gtk::runtime_thread::RuntimeThread;
use adw::{HeaderBar, prelude::*};
use gtk::Orientation;
use relm4::{
	Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, SimpleComponent, adw, gtk,
};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

const APPLICATION_ID: &str = "de.maxbruckner.baby-name-tournament";

mod backend;
mod database_list;
mod force_unwrapped_field;
mod gender_dropdown;
mod main_view;
mod name_list;
mod name_preference;
mod runtime_thread;

use crate::database;
use crate::database::views::NameWithPreference;
use crate::gtk::main_view::{MainView, MainViewInput, MainViewOutput};
use backend::Backend;

pub fn start(runtime: Runtime, database_pool: &SqlitePool) -> anyhow::Result<()> {
	let runtime_thread = RuntimeThread::start(runtime);
	let handle = runtime_thread.handle().clone();

	RelmApp::new(APPLICATION_ID)
		.with_args(vec![])
		.run::<Application>(Backend::new(database_pool.clone(), handle.clone()));

	handle.block_on(database_pool.close());

	runtime_thread.shut_down()
}

struct Application {
	main_view_controller: Controller<MainView>,
	backend: Backend,
}

#[derive(Debug)]
enum ApplicationMessage {
	NamePreferenceUpdated(NameWithPreference),
}

#[relm4::component]
impl SimpleComponent for Application {
	type Init = Backend;
	type Input = ApplicationMessage;
	type Output = ();

	view! {
		adw::ApplicationWindow {
			set_title: Some("Baby Name Tournament"),
			set_default_size: (480, 640),

			gtk::Box {
				set_orientation: Orientation::Vertical,

				HeaderBar {},

				#[local]
				main_view -> gtk::Box {},
			}
		}
	}

	fn init(backend: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let main_view_controller =
			MainView::builder()
				.launch(backend.clone())
				.forward(sender.input_sender(), |message| match message {
					MainViewOutput::NamePreferenceUpdated(name_with_preference) => {
						ApplicationMessage::NamePreferenceUpdated(name_with_preference)
					}
				});
		let main_view = main_view_controller.widget().clone();

		let model = Self {
			main_view_controller,
			backend,
		};

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		use ApplicationMessage::*;
		match message {
			NamePreferenceUpdated(NameWithPreference { name, preference, .. }) => {
				match preference {
					Some(preference) => {
						self.backend
							.block_on_future(database::upsert_name_preference(
								&name,
								preference,
								self.backend.database_pool(),
							))
							.expect("Failed to update name preference");
					}
					None => self
						.backend
						.block_on_future(database::delete_name_preference(&name, self.backend.database_pool()))
						.expect("Failed to delete name preference"),
				}
				let _ = self
					.main_view_controller
					.sender()
					.send(MainViewInput::RefreshRow { name });
			}
		}
	}
}
