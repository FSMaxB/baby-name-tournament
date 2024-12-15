use libadwaita::glib;
use libadwaita::subclass::prelude::*;
use sqlx::SqlitePool;
use std::cell::RefCell;
use tokio::runtime;

#[derive(Default, clone)]
pub struct Backend {
	database_pool: RefCell<Option<SqlitePool>>,
	runtime_handle: RefCell<Option<runtime::Handle>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Backend {
	const NAME: &'static str = "BabyNameTournamentDatabaseBackend";
	type Type = super::Backend;
	type ParentType = glib::Object;
}

impl ObjectImpl for Backend {}

impl Backend {
	pub fn initialize(&self, database_pool: SqlitePool, runtime_handle: runtime::Handle) {
		let mut database_pool_cell = self
			.database_pool
			.try_borrow_mut()
			.expect("Database pool was already borrowed.");
		let mut runtime_handle_cell = self
			.runtime_handle
			.try_borrow_mut()
			.expect("Runtime handle was already borrowed.");
		assert!(
			database_pool_cell.is_none(),
			"You must only initialize the backend once",
		);
		assert!(
			runtime_handle_cell.is_none(),
			"You must only initialize the backend once",
		);

		database_pool_cell.replace(database_pool);
		runtime_handle_cell.replace(runtime_handle);
	}

	pub fn database_pool(&self) -> SqlitePool {
		self.database_pool
			.try_borrow()
			.expect("Can't borrow database pool")
			.as_ref()
			.expect("Backend wasn't initialized")
			.clone()
	}

	pub fn runtime_handle(&self) -> runtime::Handle {
		self.runtime_handle
			.try_borrow()
			.expect("Can't borrow runtime handle")
			.as_ref()
			.expect("Backend wasn't initialized")
			.clone()
	}
}
