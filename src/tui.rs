use crate::csv_parser::Gender;
use crate::database::{list_all, Name};
use anyhow::bail;
use crossterm::cursor::Show;
use crossterm::event::Event;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures_util::{StreamExt, TryStreamExt};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState};
use ratatui::{Frame, Terminal};
use sqlx::SqlitePool;
use std::ops::Add;

pub async fn run_tui(
	database_pool: SqlitePool,
	mut terminal: Terminal<impl Backend + Send + Sync + 'static>,
) -> anyhow::Result<()> {
	let mut event_stream = EventStream::new();

	// FIXME: This currently fetches the entire list into memory (it's probably gonna be
	let all_names = list_all(Gender::Both, &database_pool).try_collect::<Vec<_>>().await?;
	if all_names.is_empty() {
		bail!("No names found");
	}
	let mut table_state = TableState::default();

	loop {
		// FIXME: Figure out how to run this without blocking the runtime!
		terminal.draw(|frame| draw(frame, &all_names, &mut table_state))?;

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
				Key(KeyEvent { code: KeyCode::Up, .. }) => {
					let selection = table_state.selected().unwrap_or_default();
					let new_selection = selection.saturating_sub(1);
					table_state.select(Some(new_selection));
					break;
				}
				Key(KeyEvent {
					code: KeyCode::Down, ..
				}) => {
					let selection = table_state.selected().unwrap_or_default();
					let new_selection = selection.saturating_add(1).min(all_names.len() - 1);
					table_state.select(Some(new_selection));
					break;
				}
				Resize(_, _) => {
					break;
				}
				_ => {}
			}
		}
	}
}

fn draw(frame: &mut Frame<impl Backend + Send + Sync + 'static>, all_names: &[Name], table_state: &mut TableState) {
	let block = Block::default()
		.borders(Borders::ALL)
		.title("Baby Name Tournament")
		.title_alignment(Alignment::Center)
		.border_type(BorderType::Rounded);

	let header_cells = ["Name", "Gender"]
		.iter()
		.map(|&header| Cell::from(header).style(Style::default().fg(Color::Red)));
	let header = Row::new(header_cells)
		.style(Style::default().bg(Color::Blue))
		.height(1)
		.bottom_margin(1);
	let rows = all_names.iter().map(|Name { name, gender }| {
		let cells = [Cell::from(name.as_str()), Cell::from(gender.as_ref())];
		Row::new(cells).height(1)
	});
	let table = Table::new(rows)
		.header(header)
		.block(block)
		.highlight_style(Style::default().add_modifier(Modifier::REVERSED))
		.highlight_symbol("> ")
		.widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

	frame.render_stateful_widget(table, frame.size(), table_state);
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
