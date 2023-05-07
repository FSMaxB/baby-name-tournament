use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list::{DatabaseListManager, DatabaseListModel, DatabaseView};
use gtk::{prelude::*, Label, PolicyType, SignalListItemFactory, SingleSelection};
use libadwaita::glib::BoxedAnyObject;
use libadwaita::gtk::Orientation;
use relm4::{gtk, RelmIterChildrenExt};
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

				#[name(name_list)]
				gtk::ListView {}
			}
		}
	}

	fn init(backend: Self::Init, _root: &Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let widgets = view_output!();

		let name_factory = SignalListItemFactory::new();

		name_factory.connect_setup(|_, list_item| {
			let name_label = Label::new(Some("<empty name>"));
			let gender_label = Label::new(Some("<empty gender>"));
			let row = gtk::Box::builder()
				.spacing(12)
				.homogeneous(true)
				.orientation(Orientation::Horizontal)
				.build();
			row.append(&name_label);
			row.append(&gender_label);
			list_item.set_child(Some(&row));
		});
		name_factory.connect_bind(|_, list_item| {
			let row = list_item
				.child()
				.expect("List item has no child")
				.downcast::<gtk::Box>()
				.expect("Incorrect type");

			let mut children = row.iter_children();
			let name_label = children
				.next()
				.expect("Missing name_label")
				.downcast::<Label>()
				.expect("Incorrect type");
			let gender_label = children
				.next()
				.expect("Missing gender_label")
				.downcast::<Label>()
				.expect("Incorrect type");

			let item = list_item
				.item()
				.expect("Missing item")
				.downcast::<BoxedAnyObject>()
				.expect("Incorrect Type");
			let name = item.borrow::<Name>();

			name_label.set_label(&name.name);
			gender_label.set_label(name.gender.as_ref());
		});

		let list_manager = DatabaseListManager::new(Gender::Both, NameListView::default());
		let list_model = DatabaseListModel::new(backend.clone(), list_manager.clone());
		let selection_model = SingleSelection::new(Some(list_model));
		widgets.name_list.set_factory(Some(&name_factory));
		widgets.name_list.set_model(Some(&selection_model));

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
