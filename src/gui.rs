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
mod name_list;
mod name_model;
mod runtime_thread;

pub fn start(runtime: Runtime, database_pool: SqlitePool) -> anyhow::Result<()> {
	let runtime_thread = RuntimeThread::start(runtime);
	let handle = runtime_thread.handle().clone();

	RelmApp::new(APPLICATION_ID).run::<Application>((handle, database_pool));

	Ok(runtime_thread.shut_down()?)
}

struct Application {
	_name_list_controller: Controller<NameList>,
}

#[relm4::component]
impl SimpleComponent for Application {
	type Init = (tokio::runtime::Handle, SqlitePool);
	type Input = ();
	type Output = ();

	view! {
		libadwaita::ApplicationWindow {
			set_title: Some("Baby Name Tournament"),
			set_default_size: (300, 100),

			gtk::Box {
				set_orientation: gtk::Orientation::Vertical,

				gtk::HeaderBar {},

				#[local]
				name_list -> gtk::Box {}
			}
		}
	}

	fn init(
		(runtime_handle, database_pool): Self::Init,
		root: &Self::Root,
		_sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let name_list_controller = NameList::builder()
			.launch((runtime_handle.clone(), database_pool.clone()))
			.detach();
		let name_list = name_list_controller.widget().clone();
		let widgets = view_output!();

		let model = Self {
			_name_list_controller: name_list_controller,
		};
		ComponentParts { model, widgets }
	}
}
