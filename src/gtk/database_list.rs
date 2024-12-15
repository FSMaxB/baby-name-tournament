mod database_list_manager;
mod database_list_model;

pub use database_list_manager::DatabaseListManager;
pub use database_list_model::DatabaseListModel;

use crate::gtk::backend::Backend;

pub trait DatabaseView: 'static {
	type Model: Model;
	type Filter: Clone;

	fn read_all(&self, backend: &Backend, filter: &Self::Filter) -> anyhow::Result<Vec<Self::Model>>;
	fn read_by_key(&self, backend: &Backend, key: &<Self::Model as Model>::Key) -> anyhow::Result<Self::Model>;
}

pub trait Model: Clone {
	type Key: PartialEq;
	fn unique_key(&self) -> &Self::Key;
}
