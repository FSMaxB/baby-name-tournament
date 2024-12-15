use crate::csv_parser::Gender;
use crate::database;
use crate::database::Name;
use crate::database::views::NameWithPreference;
use crate::gtk::backend::Backend;
use crate::gtk::database_list::{DatabaseListManager, DatabaseListModel, DatabaseView, Model};
use crate::gtk::name_list::name_list_row::{NameListRow, NameListRowInit, NameListRowInput, NameListRowOutput};
use adw::glib;
use glib::BoxedAnyObject;
use gtk::ffi::GtkListItem;
use gtk::{Orientation, PolicyType, SignalListItemFactory, prelude::*};
use relm4::{Component, ComponentParts, ComponentSender, SimpleComponent};
use relm4::{ComponentController, Controller};
use relm4::{adw, gtk};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

mod name_list_row;

pub struct NameList<VIEW: DatabaseView> {
	list_manager: DatabaseListManager<VIEW>,
	selected_names: Vec<NameWithPreference>,
	selected_names_row_controller: Controller<NameListRow>,
}

#[relm4::component(pub)]
impl<VIEW> SimpleComponent for NameList<VIEW>
where
	VIEW: DatabaseView<Model = NameWithPreference> + Default + Clone,
	VIEW::Filter: Debug,
{
	type Input = NameListInput<VIEW::Filter>;
	type Output = NameListOutput;
	type Init = (VIEW::Filter, Backend);

	view! {
		gtk::Box {
			set_homogeneous: false,
			set_orientation: Orientation::Vertical,

			gtk::ScrolledWindow {
				set_hscrollbar_policy: PolicyType::Never,
				set_propagate_natural_height: true,

				#[local_ref]
				name_list -> gtk::ListView {
					set_show_separators: true,
				}
			},

			gtk::Box {
				#[watch]
				set_visible: model.selected_names.len() > 1,
				set_orientation: Orientation::Vertical,

				gtk::Separator {},
				gtk::Separator {},
				gtk::Separator {},

				gtk::Label {
					#[watch]
					set_label: &format!("Change {} selected names", model.selected_names.len()),
				},
				#[local]
				selected_names_row -> gtk::Box {},
			}
		}
	}

	fn init(
		(initial_filter, backend): Self::Init,
		root: Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let selected_names_row_controller = NameListRow::builder()
			.launch(NameListRowInit {
				name: Name {
					name: "-".to_owned(),
					gender: Gender::Both,
				},
				preference: None,
			})
			.forward(sender.input_sender(), |message| match message {
				NameListRowOutput::NamePreferenceSet(name_with_preferences) => {
					NameListInput::MultiselectionPreferenceUpdated(name_with_preferences)
				}
			});
		let selected_names_row = selected_names_row_controller.widget().clone();

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
							name: "{none}".to_owned(),
							gender: Gender::Both,
						},
						preference: None,
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
				let name_with_preferences = item.borrow::<NameWithPreference>();

				let _ = controller
					.sender()
					.send(NameListRowInput::SetName(name_with_preferences.clone()));
			}
		});

		name_factory.connect_teardown(move |_, list_item| {
			controllers.borrow_mut().remove(&list_item.as_ptr());
		});

		let list_manager = DatabaseListManager::new(initial_filter, VIEW::default(), backend)
			.expect("Failed to initialize name list manager");
		let list_model = DatabaseListModel::new(list_manager.clone());
		let selection_model = gtk::MultiSelection::new(Some(list_model));

		selection_model.connect_selection_changed({
			let input_sender = sender.input_sender().clone();
			let list_manager = list_manager.clone();
			move |model, _, _| {
				let selected_names = list_manager
					.read_from_selection(&model.selection())
					.expect("Invalid name selected");
				let _ = input_sender.send(NameListInput::SelectionChanged(selected_names));
			}
		});
		selection_model.connect_items_changed({
			let input_sender = sender.input_sender().clone();
			let list_manager = list_manager.clone();
			move |model, _, _, _| {
				let selected_names = list_manager
					.read_from_selection(&model.selection())
					.expect("Invalid name selected");
				let _ = input_sender.send(NameListInput::SelectionChanged(selected_names));
			}
		});

		let name_list = gtk::ListView::new(Some(selection_model), Some(name_factory));

		let model = NameList {
			list_manager,
			selected_names: Vec::new(),
			selected_names_row_controller,
		};
		let widgets = view_output!();
		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NameListInput::*;
		match message {
			UpdateFilter(filter) => self
				.list_manager
				.update_filter(filter)
				.expect("Failed to update list manager"),
			NamePreferenceUpdated(name_with_preferences) => {
				let _ = sender.output(NameListOutput::NamePreferenceUpdated(name_with_preferences));
			}
			MultiselectionPreferenceUpdated(NameWithPreference { preference, .. }) => {
				// TODO: Don't destroy the existing selection when applying the value
				for NameWithPreference { name, gender, .. } in &self.selected_names {
					let _ = sender.output(NameListOutput::NamePreferenceUpdated(NameWithPreference {
						name: name.clone(),
						gender: *gender,
						preference,
					}));
				}
			}
			SelectionChanged(selected_names) => {
				self.selected_names = selected_names;
				let count = self.selected_names.len();
				if count > 1 {
					let _ = self
						.selected_names_row_controller
						.sender()
						.send(NameListRowInput::SetName(NameWithPreference {
							name: format!("{count} selected names"),
							gender: Gender::Both,
							preference: None,
						}));
				}
			}
			RefreshRow { name } => {
				self.list_manager
					.notify_updated(&name)
					.expect("Failed to update single list entry");
			}
		}
	}
}

#[derive(Debug)]
pub enum NameListInput<FILTER> {
	UpdateFilter(FILTER),
	NamePreferenceUpdated(NameWithPreference),
	MultiselectionPreferenceUpdated(NameWithPreference),
	SelectionChanged(Vec<NameWithPreference>),
	RefreshRow { name: String },
}

#[derive(Debug)]
pub enum NameListOutput {
	NamePreferenceUpdated(NameWithPreference),
}

#[derive(Clone, Default)]
pub struct NameListView;

#[derive(Clone, Debug)]
pub struct NameListViewFilter {
	pub gender: Gender,
	pub show_favorite: bool,
	pub show_nogo: bool,
	pub show_undecided: bool,
	pub name_contains: Option<String>,
}

impl Default for NameListViewFilter {
	fn default() -> Self {
		Self {
			gender: Gender::Both,
			show_favorite: true,
			show_nogo: true,
			show_undecided: true,
			name_contains: None,
		}
	}
}

impl DatabaseView for NameListView {
	type Model = NameWithPreference;
	type Filter = NameListViewFilter;

	fn read_all(
		&self,
		backend: &Backend,
		NameListViewFilter {
			gender,
			show_favorite,
			show_nogo,
			show_undecided,
			name_contains,
		}: &Self::Filter,
	) -> anyhow::Result<Vec<Self::Model>> {
		Ok(backend.block_on_future(database::views::read_all_names(
			*gender,
			*show_favorite,
			*show_nogo,
			*show_undecided,
			name_contains.as_deref(),
			backend.database_pool(),
		))?)
	}

	fn read_by_key(&self, backend: &Backend, key: &<Self::Model as Model>::Key) -> anyhow::Result<Self::Model> {
		Ok(backend.block_on_future(database::views::read_one(key, backend.database_pool()))?)
	}
}

impl Model for NameWithPreference {
	type Key = String;

	fn unique_key(&self) -> &Self::Key {
		&self.name
	}
}
