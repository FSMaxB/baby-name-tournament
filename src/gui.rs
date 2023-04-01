use crate::gui::window::Window;
use anyhow::anyhow;
use libadwaita::prelude::*;
use libadwaita::Application;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

const APPLICATION_ID: &str = "de.maxbruckner.baby-name-tournament";

pub mod name_list;
pub mod name_model;
pub mod window;

pub fn start(_runtime: Runtime, _database_pool: SqlitePool) -> anyhow::Result<()> {
	let application = Application::builder().application_id(APPLICATION_ID).build();

	application.connect_activate(build_ui);

	let exit_code = application.run_with_args::<&str>(&[]);
	match exit_code.value() {
		0 => Ok(()),
		code => Err(anyhow!("GTK application finished with exit code {code}")),
	}
}

fn build_ui(application: &Application) {
	Window::new(application).show();
}
