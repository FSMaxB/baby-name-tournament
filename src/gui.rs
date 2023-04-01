use gtk::ApplicationWindow;
use libadwaita::gtk;
use libadwaita::prelude::*;
use libadwaita::Application;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;

const APPLICATION_ID: &str = "de.maxbruckner.baby-name-tournament";

pub fn start(_runtime: Runtime, _database_pool: SqlitePool) {
	let application = Application::builder().application_id(APPLICATION_ID).build();

	application.connect_activate(build_ui);

	application.run_with_args::<&str>(&[]);
}

fn build_ui(application: &Application) {
	let window = ApplicationWindow::builder().title("Baby Name Tournament").build();
	window.set_application(Some(application));

	window.show();
}
