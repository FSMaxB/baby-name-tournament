use crate::csv_parser::Gender;
use crate::database;
use crate::gui::name_model::NameModel;
use futures_util::TryStreamExt;
use once_cell::unsync::OnceCell;
use sqlx::SqlitePool;
use std::future::Future;
use std::ops::Deref;
use std::rc::Rc;
use tokio::runtime;

#[derive(Default, Clone)]
pub struct Backend {
	inner: OnceCell<Rc<BackendInner>>,
}

struct BackendInner {
	database_pool: SqlitePool,
	runtime_handle: runtime::Handle,
}

impl Backend {
	pub fn new(database_pool: SqlitePool, runtime_handle: runtime::Handle) -> Self {
		Self {
			inner: OnceCell::with_value(Rc::new(BackendInner {
				database_pool,
				runtime_handle,
			})),
		}
	}

	pub fn initialize(&self, backend: Self) {
		let Some(inner) = backend.inner.into_inner() else {
			panic!("Tried to initialize backend from uninitialized backend");
		};

		assert!(
			self.inner.get().is_none(),
			"Tried to initialize backend that was already initialized"
		);
		let _ = self.inner.set(inner);
	}

	pub fn database_pool(&self) -> &SqlitePool {
		&self.inner().database_pool
	}

	pub fn runtime_handle(&self) -> &runtime::Handle {
		&self.inner().runtime_handle
	}

	// FIXME: This blocks the main loop and doesn't do any error handling
	pub fn list_all_names(&self) -> Vec<NameModel> {
		self.block_on_future(async {
			database::list_all(Gender::Both, self.database_pool())
				.map_ok(NameModel::from)
				.try_collect()
				.await
		})
		.expect("Database error")
	}

	pub fn block_on_future<FUTURE>(&self, future: FUTURE) -> FUTURE::Output
	where
		FUTURE: Future,
	{
		self.runtime_handle().block_on(future)
	}

	fn inner(&self) -> &BackendInner {
		self.inner
			.get()
			.unwrap_or_else(|| panic!("Backend needs to be initialized first"))
			.deref()
	}
}
