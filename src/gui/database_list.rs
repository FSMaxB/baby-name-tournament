mod database_list_manager;
mod database_list_model;

pub use database_list_manager::DatabaseListManager;
pub use database_list_model::DatabaseListModel;

use crate::gui::backend::Backend;

pub trait DatabaseView: 'static {
	type Model;
	type Filter: Clone;

	fn read_at_offset(&self, backend: &Backend, filter: &Self::Filter, offset: u32) -> anyhow::Result<Self::Model>;
	fn count(&self, backend: &Backend, filter: &Self::Filter) -> u32;
}
