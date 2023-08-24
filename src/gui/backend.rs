use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use futures_util::TryStreamExt;
use sqlx::SqlitePool;
use std::cell::OnceCell;
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
			inner: Rc::new(BackendInner {
				database_pool,
				runtime_handle,
			})
			.into(),
		}
	}

	pub fn initialize(&self, backend: Self) {
		let Some(inner) = backend.inner.into_inner() else {
			panic!("Tried to initialize backend from uninitialized backend");
		};

		if self.inner.set(inner).is_err() {
			panic!("Tried to initialize backend that was already initialized");
		}
	}

	pub fn database_pool(&self) -> &SqlitePool {
		&self.inner().database_pool
	}

	pub fn runtime_handle(&self) -> &runtime::Handle {
		&self.inner().runtime_handle
	}

	pub async fn list_all_names(&self) -> Vec<Name> {
		let database_pool = self.database_pool().clone();
		self.run_future(async move { database::list_all(Gender::Both, &database_pool).try_collect().await })
			.await
			.expect("Database error")
	}

	pub async fn run_future<FUTURE>(&self, future: FUTURE) -> FUTURE::Output
	where
		FUTURE: Future + Send + 'static,
		FUTURE::Output: Send,
	{
		self.runtime_handle()
			.spawn(future)
			.await
			.expect("Running future failed")
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
