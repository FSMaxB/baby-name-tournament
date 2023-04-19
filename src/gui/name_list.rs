use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list_model::{DatabaseListModel, DatabaseView, DatabaseViewExt};
use crate::gui::name_model::NameModel;
use async_trait::async_trait;
use gtk::{prelude::*, ColumnViewColumn, Label, PolicyType, SignalListItemFactory, SingleSelection, Widget};
use once_cell::unsync;
use relm4::gtk;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::cell::Cell;
use std::rc::Rc;

pub struct NameList {
	list_view: NameListView,
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

		let list_view = NameListView::default();
		let list_model = DatabaseListModel::new(backend.clone(), list_view.clone());
		let selection_model = SingleSelection::new(Some(list_model));

		widgets.column_view.set_model(Some(&selection_model));
		widgets.column_view.append_column(&name_column);
		widgets.column_view.append_column(&gender_column);

		let model = NameList { list_view, backend };
		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.list_view.update_filter(&self.backend, message);
	}
}

#[derive(Clone)]
struct NameListView {
	gender: Rc<Cell<Gender>>,
	// FIXME: Pull count and callback out into a wrapper type to prevent incorrect state management!
	count: Rc<Cell<u32>>,
	callback: Rc<unsync::OnceCell<Box<dyn Fn(u32, u32)>>>,
}

impl Default for NameListView {
	fn default() -> Self {
		Self {
			gender: Rc::new(Cell::new(Gender::Both)),
			count: Rc::new(Cell::new(0)),
			callback: Default::default(),
		}
	}
}

#[async_trait]
impl DatabaseView for NameListView {
	type RustModel = Name;
	type Filter = Gender;
	type GObjectModel = NameModel;

	fn read_at_offset(&self, backend: &Backend, offset: u32) -> anyhow::Result<Self::RustModel> {
		let gender = self.gender.get();
		let model = backend.block_on_future(database::read_name_at_offset(offset, gender, backend.database_pool()))?;
		Ok(model)
	}

	fn count(&self, backend: &Backend) -> u32 {
		let gender = self.gender.get();
		backend
			.block_on_future(database::count_names(gender, backend.database_pool()))
			.expect("Failed to count names") as u32
	}

	fn update_filter(&self, backend: &Backend, filter: Self::Filter) {
		self.gender.set(filter);
		self.notify_changed(backend);
	}

	fn items_changed_callback_storage(&self) -> &unsync::OnceCell<Box<dyn Fn(u32, u32)>> {
		&self.callback
	}

	fn count_storage(&self) -> &Cell<u32> {
		&self.count
	}
}
