use crate::csv_parser::Gender;
use crate::database;
use crate::database::views::NameWithPreferences;
use crate::database::{Name, NamePreference};
use crate::gui::backend::Backend;
use crate::gui::database_list::{DatabaseListManager, DatabaseListModel, DatabaseView};
use crate::gui::name_list::name_list_row::{NameListRow, NameListRowInit, NameListRowInput, NameListRowOutput};
use gtk::{prelude::*, PolicyType, SignalListItemFactory, SingleSelection};
use libadwaita::glib::BoxedAnyObject;
use libadwaita::gtk;
use libadwaita::gtk::ffi::GtkListItem;
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent};
use relm4::{ComponentController, Controller};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

mod name_list_row;

pub struct NameList<VIEW: DatabaseView> {
	list_manager: DatabaseListManager<VIEW>,
	backend: Backend,
}

#[relm4::component(pub)]
impl<VIEW> SimpleComponent for NameList<VIEW>
where
	VIEW: DatabaseView<Model = NameWithPreferences> + Default + Clone,
	VIEW::Filter: Debug,
{
	type Input = NameListInput<VIEW::Filter>;
	type Output = NameListOutput;
	type Init = (VIEW::Filter, Backend);

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

	fn init(
		(initial_filter, backend): Self::Init,
		_root: &Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let widgets = view_output!();

		let name_factory = SignalListItemFactory::new();

		// FIXME: Find a better way than the pointer of the root widget to identify which component is hooked up to which GtkListItem
		let controllers = Rc::new(RefCell::new(HashMap::<*mut GtkListItem, Controller<NameListRow>>::new()));

		name_factory.connect_setup({
			let sender = sender.clone();
			let controllers = controllers.clone();
			move |_, list_item| {
				let controller = NameListRow::builder()
					.launch(NameListRowInit {
						name: Name {
							name: "<none>".to_owned(),
							gender: Gender::Both,
						},
						mother_preference: NamePreference::Neutral,
						father_preference: NamePreference::Neutral,
					})
					.forward(sender.input_sender(), |output| match output {
						NameListRowOutput::NamePreferenceSet(name_with_preferences) => {
							NameListInput::NamePreferenceUpdated(name_with_preferences)
						}
					});
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
				let name_with_preferences = item.borrow::<NameWithPreferences>();

				let _ = controller
					.sender()
					.send(NameListRowInput::SetName(name_with_preferences.clone()));
			}
		});

		name_factory.connect_teardown(move |_, list_item| {
			controllers.borrow_mut().remove(&list_item.as_ptr());
		});

		let list_manager = DatabaseListManager::new(initial_filter, VIEW::default());
		let list_model = DatabaseListModel::new(backend.clone(), list_manager.clone());
		let selection_model = SingleSelection::new(Some(list_model));
		widgets.name_list.set_factory(Some(&name_factory));
		widgets.name_list.set_model(Some(&selection_model));

		widgets.name_list.connect_activate({
			let list_manager = list_manager.clone();
			let backend = backend.clone();
			let input_sender = sender.input_sender().clone();
			move |_, position| {
				let Ok(NameWithPreferences {name, gender, ..}) = list_manager.read_at_offset(&backend, position) else {
					return;
				};
				let _ = input_sender.send(NameListInput::NameSelected(Name { name, gender }));
			}
		});

		let model = NameList { list_manager, backend };
		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NameListInput::*;
		match message {
			UpdateFilter(filter) => self.list_manager.update_filter(&self.backend, filter),
			NameSelected(name) => {
				let _ = sender.output_sender().send(NameListOutput::NameSelected(name));
			}
			NamePreferenceUpdated(name_with_preferences) => {
				let _ = sender
					.output_sender()
					.send(NameListOutput::NamePreferenceUpdated(name_with_preferences));
			}
			Refresh => {
				self.list_manager.notify_changed(&self.backend);
			}
		}
	}
}

#[derive(Debug)]
pub enum NameListInput<FILTER> {
	UpdateFilter(FILTER),
	NameSelected(Name),
	NamePreferenceUpdated(NameWithPreferences),
	Refresh,
}

#[derive(Debug)]
pub enum NameListOutput {
	NameSelected(Name),
	NamePreferenceUpdated(NameWithPreferences),
}

#[derive(Clone, Default)]
pub struct NameListView;

#[derive(Clone, Debug)]
pub struct NameListViewFilter {
	pub gender: Gender,
	pub show_favorite: bool,
	pub show_nogo: bool,
	pub show_neutral: bool,
	pub name_contains: Option<String>,
}

impl Default for NameListViewFilter {
	fn default() -> Self {
		Self {
			gender: Gender::Both,
			show_favorite: true,
			show_nogo: true,
			show_neutral: true,
			name_contains: None,
		}
	}
}

impl DatabaseView for NameListView {
	type Model = NameWithPreferences;
	type Filter = NameListViewFilter;

	fn read_at_offset(&self, backend: &Backend, filter: &Self::Filter, offset: u32) -> anyhow::Result<Self::Model> {
		let model = backend.block_on_future(database::views::read_name_at_offset(
			offset,
			filter.gender,
			filter.show_favorite,
			filter.show_nogo,
			filter.show_neutral,
			filter.name_contains.as_deref(),
			backend.database_pool(),
		))?;
		Ok(model)
	}

	fn count(&self, backend: &Backend, filter: &Self::Filter) -> u32 {
		backend
			.block_on_future(database::views::count_names(
				filter.gender,
				filter.show_favorite,
				filter.show_nogo,
				filter.show_neutral,
				filter.name_contains.as_deref(),
				backend.database_pool(),
			))
			.expect("Failed to count names") as u32
	}
}
