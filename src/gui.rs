use crate::gui::backend::Backend;
use crate::gui::runtime_thread::RuntimeThread;
use libadwaita::gtk;
use libadwaita::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, SimpleComponent};
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
	runtime_handle: tokio::runtime::Handle,
	database_pool: SqlitePool,
}

#[relm4::component]
impl SimpleComponent for Application {
	type Init = (tokio::runtime::Handle, SqlitePool);
	type Input = ();
	type Output = ();

	view! {
		gtk::Window {
			set_title: Some("Baby Name Tournament"),
			set_default_size: (300, 100),

			gtk::Box {
				set_orientation: gtk::Orientation::Vertical,

				#[name(name_list)]
				name_list::NameList {}
			}
		}
	}

	fn init(
		(runtime_handle, database_pool): Self::Init,
		root: &Self::Root,
		_sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let backend = Backend::new(database_pool.clone(), runtime_handle.clone());
		let model = Self {
			runtime_handle,
			database_pool,
		};
		let widgets = view_output!();

		widgets.name_list.initialize(backend);

		ComponentParts { model, widgets }
	}

	fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {}
}
