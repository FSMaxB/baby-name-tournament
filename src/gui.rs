use crate::gui::backend::Backend;
use crate::gui::runtime_thread::RuntimeThread;
use crate::gui::window::Window;
use anyhow::anyhow;
use libadwaita::prelude::*;
use libadwaita::Application;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

const APPLICATION_ID: &str = "de.maxbruckner.baby-name-tournament";

mod backend;
mod database_list_model;
mod force_unwrapped_field;
mod name_list;
mod name_model;
mod runtime_thread;
mod window;

pub fn start(runtime: Runtime, database_pool: SqlitePool) -> anyhow::Result<()> {
	let runtime_thread = RuntimeThread::start(runtime);

	let application = Application::builder().application_id(APPLICATION_ID).build();
	application.connect_activate({
		let runtime_handle = runtime_thread.handle().clone();
		move |application| {
			let backend = Backend::new(database_pool.clone(), runtime_handle.clone());
			Window::new(application, backend).show()
		}
	});

	let exit_code = application.run_with_args::<&str>(&[]);

	runtime_thread.shut_down()?;

	match exit_code.value() {
		0 => Ok(()),
		code => Err(anyhow!("GTK application finished with exit code {code}")),
	}
}
