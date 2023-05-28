use async_channel::bounded;
use derive_more::From;
use futures_util::Stream;
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub fn stream_blocking_iterator<ITERATOR, ITEM>(iterator: ITERATOR) -> impl Stream<Item = ITEM>
where
	ITERATOR: IntoIterator<Item = ITEM> + Send + 'static,
	ITEM: Send + 'static,
{
	let (sender, receiver) = bounded(32);

	tokio::task::spawn_blocking(move || {
		for item in iterator {
			if sender.send_blocking(item).is_err() {
				break;
			}
		}
	});

	receiver
}

#[derive(From)]
pub struct PrettyPrintedDuration(Duration);

impl Display for PrettyPrintedDuration {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let seconds = self.0.as_secs();

		let hours = seconds / 3600;
		if hours > 0 {
			write!(formatter, "{hours}h ")?;
		}

		let minutes = (seconds % 3600) / 60;
		if minutes > 0 {
			write!(formatter, "{minutes}min ")?;
		}

		write!(formatter, "{}s", seconds % 60)
	}
}
