use crate::csv_parser::Gender;
use crate::gui::gender_dropdown::GenderDropdown;
use crate::gui::name_list::NameList;
use crate::gui::runtime_thread::RuntimeThread;
use libadwaita::prelude::*;
use relm4::{
	gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, SimpleComponent,
};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

const APPLICATION_ID: &str = "de.maxbruckner.baby-name-tournament";

mod backend;
mod database_list_model;
mod force_unwrapped_field;
mod gender_dropdown;
mod name_list;
mod name_model;
mod runtime_thread;

use backend::Backend;

pub fn start(runtime: Runtime, database_pool: SqlitePool) -> anyhow::Result<()> {
	let runtime_thread = RuntimeThread::start(runtime);
	let handle = runtime_thread.handle().clone();

	RelmApp::new(APPLICATION_ID).run::<Application>(Backend::new(database_pool, handle));

	runtime_thread.shut_down()
}

struct Application {
	name_list_controller: Controller<NameList>,
	_gender_dropdown_controller: Controller<GenderDropdown>,
}

#[derive(Debug)]
enum ApplicationMessage {
	GenderSelected(Gender),
}

#[relm4::component]
impl SimpleComponent for Application {
	type Init = Backend;
	type Input = ApplicationMessage;
	type Output = ();

	view! {
		libadwaita::ApplicationWindow {
			set_title: Some("Baby Name Tournament"),
			set_default_size: (300, 100),

			gtk::Box {
				set_orientation: gtk::Orientation::Vertical,

				gtk::HeaderBar {},

				#[local_ref]
				gender_dropdown -> gtk::DropDown {},

				#[local_ref]
				name_list -> gtk::Box {}
			}
		}
	}

	fn init(backend: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let name_list_controller = NameList::builder().launch(backend).detach();
		let name_list = name_list_controller.widget();

		let gender_dropdown_controller = GenderDropdown::builder()
			.launch(())
			.forward(sender.input_sender(), ApplicationMessage::GenderSelected);
		let gender_dropdown = gender_dropdown_controller.widget();

		let widgets = view_output!();

		let model = Self {
			name_list_controller,
			_gender_dropdown_controller: gender_dropdown_controller,
		};
		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		use ApplicationMessage::*;
		match message {
			GenderSelected(gender) => {
				self.name_list_controller
					.sender()
					.send(gender)
					.expect("Failed to send gender");
			}
		}
	}
}
