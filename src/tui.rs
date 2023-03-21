use crossterm::cursor::Show;
use crossterm::event::Event;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures_util::StreamExt;
use ratatui::backend::Backend;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{Frame, Terminal};

pub async fn run_tui(mut terminal: Terminal<impl Backend + Send + Sync + 'static>) -> anyhow::Result<()> {
	let mut event_stream = EventStream::new();

	loop {
		// FIXME: Figure out how to run this without blocking the runtime!
		terminal.draw(|frame| draw(frame))?;

		loop {
			let event = match event_stream.next().await {
				Some(event) => event?,
				None => return Ok(()),
			};

			use Event::*;
			match event {
				Key(KeyEvent {
					code: KeyCode::Char('q'),
					..
				}) => {
					return Ok(());
				}
				Resize(_, _) => {
					break;
				}
				_ => {}
			}
		}
	}
}

fn draw(frame: &mut Frame<impl Backend + Send + Sync + 'static>) {
	let block = Block::default()
		.borders(Borders::ALL)
		.title("Baby Name Tournament")
		.title_alignment(Alignment::Center)
		.border_type(BorderType::Rounded);
	frame.render_widget(block, frame.size());
}

#[must_use = "If you drop this, the TUI mode is exited"]
pub struct TuiMode {
	_private: (),
}

impl TuiMode {
	pub fn enable() -> anyhow::Result<Self> {
		enable_raw_mode()?;
		execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
		Ok(Self { _private: () })
	}
}

impl Drop for TuiMode {
	fn drop(&mut self) {
		if let Err(error) = disable_raw_mode() {
			eprintln!("Failed to disable raw mode: {error}");
		}
		if let Err(error) = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture, Show) {
			eprintln!("Failed to leave alternate screen, disable mouse capture and show the cursor: {error}");
		}
	}
}
