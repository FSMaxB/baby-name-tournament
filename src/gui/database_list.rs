mod database_list_manager;
mod database_list_model;

pub use database_list_manager::DatabaseListManager;
pub use database_list_model::DatabaseListModel;

use crate::gui::backend::Backend;

pub trait DatabaseView: 'static {
	type Model: Clone;
	type Filter: Clone;

	fn read_all(&self, backend: &Backend, filter: &Self::Filter) -> anyhow::Result<Vec<Self::Model>>;
}
