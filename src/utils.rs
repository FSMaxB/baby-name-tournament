use async_channel::bounded;
use futures_util::Stream;

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
