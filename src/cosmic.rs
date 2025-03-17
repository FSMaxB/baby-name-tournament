use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length};
use cosmic::{Application, ApplicationExt, Element, task, widget};
use sqlx::SqlitePool;

pub fn start(database_pool: SqlitePool) -> anyhow::Result<()> {
	let settings =
		cosmic::app::Settings::default().size_limits(cosmic::iced::Limits::NONE.min_width(480.0).min_height(640.0));

	cosmic::app::run::<AppModel>(settings, database_pool)?;

	Ok(())
}

pub struct AppModel {
	core: Core,
	counter: i8,
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
		}

		Task::none()
	}

	fn view(&self) -> Element<Self::Message> {
		let increase_button = widget::button::standard("+").on_press(Message::Increase);
		let count = widget::text::title1(self.counter.to_string())
			.width(Length::Fill)
			.align_x(Alignment::Center);
		let decrease_button = widget::button::standard("-").on_press(Message::Decrease);

		let counter_assembly = widget::column()
			.push(increase_button)
			.push(count)
			.push(decrease_button)
			.width(Length::Fixed(50.0));

		widget::container(counter_assembly)
			.width(Length::Fill)
			.height(Length::Fill)
			.align_x(Alignment::Center)
			.align_y(Alignment::Center)
			.into()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
	Increase,
	Decrease,
	ClosePool,
	Nothing,
}
