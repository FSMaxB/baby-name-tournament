use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list_model::{DatabaseListModel, DatabaseView};
use crate::gui::name_model::NameModel;
use async_trait::async_trait;
use gtk::{prelude::*, ColumnViewColumn, Label, PolicyType, SignalListItemFactory, SingleSelection, Widget};
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use sqlx::SqlitePool;

pub struct NameList;

#[relm4::component(pub)]
impl SimpleComponent for NameList {
	type Input = ();
	type Output = ();
	type Init = (tokio::runtime::Handle, SqlitePool);

	view! {
		gtk::Box {
			set_homogeneous: true,

			gtk::ScrolledWindow {
				set_hscrollbar_policy: PolicyType::Never,
				set_propagate_natural_height: true,

				#[name(column_view)]
				gtk::ColumnView {
					set_reorderable: false,
					set_show_column_separators: true,
					set_show_row_separators: true,
				}
			}
		}
	}

	fn init(
		(runtime_handle, database_pool): Self::Init,
		_root: &Self::Root,
		_sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let widgets = view_output!();

		let name_factory = SignalListItemFactory::new();
		let gender_factory = SignalListItemFactory::new();

		name_factory.connect_setup(|_, list_item| {
			let name_label = Label::new(Some("<empty name>"));
			list_item.set_child(Some(&name_label));
			list_item
				.property_expression("item")
				.chain_property::<NameModel>("name")
				.bind(&name_label, "label", Widget::NONE);
		});
		gender_factory.connect_setup(|_, list_item| {
			let gender_label = Label::new(Some("<empty gender>"));
			list_item.set_child(Some(&gender_label));
			list_item
				.property_expression("item")
				.chain_property::<NameModel>("gender")
				.bind(&gender_label, "label", Widget::NONE);
		});

		let name_column = ColumnViewColumn::builder()
			.title("Name")
			.resizable(true)
			.factory(&name_factory)
			.build();
		let gender_column = ColumnViewColumn::builder()
			.title("Gender")
			.resizable(true)
			.factory(&gender_factory)
			.build();

		let list_view = NameListView;
		let list_model = DatabaseListModel::new(Backend::new(database_pool.clone(), runtime_handle.clone()), list_view);
		let selection_model = SingleSelection::new(Some(list_model));

		widgets.column_view.set_model(Some(&selection_model));
		widgets.column_view.append_column(&name_column);
		widgets.column_view.append_column(&gender_column);

		ComponentParts {
			model: NameList,
			widgets,
		}
	}
}

struct NameListView;

#[async_trait]
impl DatabaseView for NameListView {
	type RustModel = Name;
	type GObjectModel = NameModel;

	async fn read_at_offset(database_pool: &SqlitePool, offset: u32) -> anyhow::Result<Self::RustModel> {
		Ok(database::read_name_at_offset(offset, Gender::Both, database_pool).await?)
	}

	async fn count(database_pool: &SqlitePool) -> u32 {
		database::count_names(Gender::Both, database_pool)
			.await
			.expect("Failed to count names") as u32
	}
}
