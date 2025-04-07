use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::Icon;
use cosmic::widget::table::{Entity, ItemCategory, ItemInterface, MultiSelectModel, MultiSelectTableView};
use cosmic::{Application, ApplicationExt, Element, task, widget};
use sqlx::SqlitePool;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

pub fn start(database_pool: SqlitePool) -> anyhow::Result<()> {
	let settings =
		cosmic::app::Settings::default().size_limits(cosmic::iced::Limits::NONE.min_width(480.0).min_height(640.0));

	cosmic::app::run::<AppModel>(settings, database_pool)?;

	Ok(())
}

pub struct AppModel {
	core: Core,
	counter: i8,
	next_list_entry: usize,
	list_model: MultiSelectModel<ListItem, ListCategory>,
	database_pool: SqlitePool,
}

impl Application for AppModel {
	type Executor = cosmic::executor::Default;
	type Flags = SqlitePool;
	type Message = Message;

	const APP_ID: &'static str = "de.maxbruckner.baby-name-tournament";

	fn core(&self) -> &Core {
		&self.core
	}

	fn core_mut(&mut self) -> &mut Core {
		&mut self.core
	}

	fn init(core: Core, database_pool: Self::Flags) -> (Self, Task<Self::Message>) {
		let mut model = Self {
			core,
			counter: 0,
			next_list_entry: 0,
			list_model: MultiSelectModel::new(vec![ListCategory]),
			database_pool,
		};

		let title = "Baby Name Tournament".to_owned();
		model.set_header_title(title.clone());
		let task = model.set_window_title(title.clone());

		(model, task)
	}

	fn on_close_requested(&self, id: Id) -> Option<Self::Message> {
		if id != self.core.main_window_id()? {
			return None;
		}

		Some(Message::ClosePool)
	}

	fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
		use Message::*;
		match message {
			Increase => {
				self.counter = self.counter.saturating_add(1);
			}
			Decrease => {
				self.counter = self.counter.saturating_sub(1);
			}
			ClosePool => {
				return task::future({
					let database_pool = self.database_pool.clone();
					async move {
						// TODO: Somehow make sure this actually runs, because it apparently doesn't right now
						database_pool.close().await;
						Nothing
					}
				});
			}
			Nothing => {}
			AddListEntry => {
				let _ = self.list_model.insert(self.next_list_entry.into());
				self.next_list_entry += 1;
			}
			ListEntrySelected(selected_entry) => {
				self.list_model.activate(selected_entry);
				println!("Selected items:");
				for selected in self.list_model.active() {
					let ListItem(count) = self.list_model.item(selected).expect("Invalid item selected");
					println!("{count}");
				}
			}
		}

		Task::none()
	}

	fn view(&self) -> Element<Self::Message> {
		let increase_button = widget::button::standard("+").on_press(Message::Increase);
		let count = widget::text::title1(self.counter.to_string())
			.width(Length::Fill)
			.align_x(Alignment::Center);
		let decrease_button = widget::button::standard("-").on_press(Message::Decrease);

		let add_list_entry_button = widget::button::standard("Add list entry").on_press(Message::AddListEntry);

		let list = MultiSelectTableView::new(&self.list_model).on_item_left_click(Message::ListEntrySelected);

		let counter_assembly = widget::column()
			.push(increase_button)
			.push(count)
			.push(decrease_button)
			.push(add_list_entry_button)
			.push(list)
			.width(Length::Fixed(50.0));

		widget::container(counter_assembly)
			.width(Length::Fill)
			.height(Length::Fill)
			.align_x(Alignment::Center)
			.align_y(Alignment::Center)
			.into()
	}
}

#[derive(Debug, Clone)]
pub enum Message {
	Increase,
	Decrease,
	ClosePool,
	Nothing,
	AddListEntry,
	ListEntrySelected(Entity),
}

struct ListItem(usize);

impl ItemInterface<ListCategory> for ListItem {
	fn get_icon(&self, _: ListCategory) -> Option<Icon> {
		None
	}

	fn get_text(&self, _: ListCategory) -> Cow<'static, str> {
		self.0.to_string().into()
	}

	fn compare(&self, other: &Self, _: ListCategory) -> Ordering {
		self.0.cmp(&other.0)
	}
}

impl From<usize> for ListItem {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct ListCategory;

impl Display for ListCategory {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "List")
	}
}

impl ItemCategory for ListCategory {
	fn width(&self) -> Length {
		Length::Fixed(100.0)
	}
}
