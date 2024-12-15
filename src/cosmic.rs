use cosmic::app::{Core, Task};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Length;
use cosmic::{widget, Application, Apply, Element};
use sqlx::SqlitePool;

pub fn start(_database_pool: &SqlitePool) -> anyhow::Result<()> {
	let settings =
		cosmic::app::Settings::default().size_limits(cosmic::iced::Limits::NONE.min_width(480.0).min_height(640.0));

	cosmic::app::run::<AppModel>(settings, ())?;

	Ok(())
}

pub struct AppModel {
	core: Core,
}

impl Application for AppModel {
	type Executor = cosmic::executor::Default;
	type Flags = ();
	type Message = ();

	const APP_ID: &'static str = "de.maxbruckner.baby-name-tournament";

	fn core(&self) -> &Core {
		&self.core
	}

	fn core_mut(&mut self) -> &mut Core {
		&mut self.core
	}

	fn init(core: Core, (): Self::Flags) -> (Self, Task<Self::Message>) {
		let model = Self { core };

		(model, Task::none())
	}

	fn view(&self) -> Element<Self::Message> {
		widget::text::title1("Hello, world!")
			.apply(widget::container)
			.width(Length::Fill)
			.height(Length::Fill)
			.align_x(Horizontal::Center)
			.align_y(Vertical::Center)
			.into()
	}
}
