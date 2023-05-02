use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list::{DatabaseListManager, DatabaseListModel, DatabaseView};
use gtk::{prelude::*, ColumnViewColumn, Label, PolicyType, SignalListItemFactory, SingleSelection};
use libadwaita::glib::BoxedAnyObject;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub struct NameList {
	list_manager: DatabaseListManager<NameListView>,
	backend: Backend,
}

#[relm4::component(pub)]
impl SimpleComponent for NameList {
	type Input = Gender;
	type Output = ();
	type Init = Backend;

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

	fn init(backend: Self::Init, _root: &Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let widgets = view_output!();

		let name_factory = SignalListItemFactory::new();
		let gender_factory = SignalListItemFactory::new();

		name_factory.connect_setup(|_, list_item| {
			let name_label = Label::new(Some("<empty name>"));
			list_item.set_child(Some(&name_label));
		});
		name_factory.connect_bind(|_, list_item| {
			let name_label = list_item
				.child()
				.expect("List item has no child")
				.downcast::<Label>()
				.expect("Not a label");
			let item = list_item
				.item()
				.expect("ListItem has no item")
				.downcast::<BoxedAnyObject>()
				.expect("Incorrect type");
			let name = item.borrow::<Name>();

			name_label.set_label(&name.name);
		});
		gender_factory.connect_setup(|_, list_item| {
			let gender_label = Label::new(Some("<empty gender>"));
			list_item.set_child(Some(&gender_label));
		});
		gender_factory.connect_bind(|_, list_item| {
			let gender_label = list_item
				.child()
				.expect("List item has no child")
				.downcast::<Label>()
				.expect("Not a label");
			let item = list_item
				.item()
				.expect("ListItem has no item")
				.downcast::<BoxedAnyObject>()
				.expect("Incorrect type");
			let name = item.borrow::<Name>();

			gender_label.set_label(name.gender.as_ref());
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

		let list_manager = DatabaseListManager::new(Gender::Both, NameListView::default());
		let list_model = DatabaseListModel::new(backend.clone(), list_manager.clone());
		let selection_model = SingleSelection::new(Some(list_model));

		widgets.column_view.set_model(Some(&selection_model));
		widgets.column_view.append_column(&name_column);
		widgets.column_view.append_column(&gender_column);

		let model = NameList { list_manager, backend };
		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.list_manager.update_filter(&self.backend, message);
	}
}

#[derive(Clone, Default)]
struct NameListView;

impl DatabaseView for NameListView {
	type Model = Name;
	type Filter = Gender;

	fn read_at_offset(&self, backend: &Backend, filter: Self::Filter, offset: u32) -> anyhow::Result<Self::Model> {
		let model = backend.block_on_future(database::read_name_at_offset(offset, filter, backend.database_pool()))?;
		Ok(model)
	}

	fn count(&self, backend: &Backend, filter: Self::Filter) -> u32 {
		backend
			.block_on_future(database::count_names(filter, backend.database_pool()))
			.expect("Failed to count names") as u32
	}
}
