use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::gui::backend::Backend;
use crate::gui::database_list::{DatabaseListManager, DatabaseListModel, DatabaseView};
use crate::gui::name_list::name_list_row::NameListRow;
use gtk::{prelude::*, PolicyType, SignalListItemFactory, SingleSelection};
use libadwaita::glib::BoxedAnyObject;
use libadwaita::gtk;
use libadwaita::gtk::ffi::GtkListItem;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent};
use relm4::{ComponentController, Controller};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod name_list_row;

pub struct NameList {
	list_manager: DatabaseListManager<NameListView>,
	backend: Backend,
}

#[relm4::component(pub)]
impl SimpleComponent for NameList {
	type Input = NameListInput;
	type Output = Name;
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

	fn init(backend: Self::Init, _root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let widgets = view_output!();

		let name_factory = SignalListItemFactory::new();

		// FIXME: Find a better way than the pointer of the root widget to identify which component is hooked up to which GtkListItem
		let controllers = Rc::new(RefCell::new(HashMap::<*mut GtkListItem, Controller<NameListRow>>::new()));

		name_factory.connect_setup({
			let sender = sender.clone();
			let controllers = controllers.clone();
			move |_, list_item| {
				let controller = NameListRow::builder()
					.launch(Name {
						name: "<none>".to_owned(),
						gender: Gender::Both,
					})
					.forward(sender.input_sender(), |_| unreachable!());
				list_item.set_child(Some(controller.widget()));
				controllers.borrow_mut().insert(list_item.as_ptr(), controller);
			}
		});
		name_factory.connect_bind({
			let controllers = controllers.clone();
			move |_, list_item| {
				let controllers = controllers.borrow();
				let controller = controllers
					.get(&list_item.as_ptr())
					.expect("No controller for this list item");

				let item = list_item
					.item()
					.expect("Missing item")
					.downcast::<BoxedAnyObject>()
					.expect("Incorrect Type");
				let name = item.borrow::<Name>();

				let _ = controller.sender().send(name.clone());
			}
		});

		name_factory.connect_teardown({
			let controllers = controllers.clone();
			move |_, list_item| {
				controllers.borrow_mut().remove(&list_item.as_ptr());
			}
		});

		let list_manager = DatabaseListManager::new(Gender::Both, NameListView::default());
		let list_model = DatabaseListModel::new(backend.clone(), list_manager.clone());
		let selection_model = SingleSelection::new(Some(list_model));
		widgets.name_list.set_factory(Some(&name_factory));
		widgets.name_list.set_model(Some(&selection_model));

		widgets.name_list.connect_activate({
			let list_manager = list_manager.clone();
			let backend = backend.clone();
			let input_sender = sender.input_sender().clone();
			move |_, position| {
				let Ok(name) = list_manager.read_at_offset(&backend, position) else {
					return;
				};
				let _ = input_sender.send(NameListInput::NameSelected(name));
			}
		});

		let model = NameList { list_manager, backend };
		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NameListInput::*;
		match message {
			GenderFiltered(gender) => self.list_manager.update_filter(&self.backend, gender),
			NameSelected(name) => {
				let _ = sender.output_sender().send(name);
			}
		}
	}
}

#[derive(Debug)]
pub enum NameListInput {
	GenderFiltered(Gender),
	NameSelected(Name),
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
