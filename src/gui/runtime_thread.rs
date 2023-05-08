use anyhow::anyhow;
use std::time::Duration;
use tokio::runtime::{self, Runtime};
use tokio::sync::oneshot;

pub struct RuntimeThread {
	join_handle: std::thread::JoinHandle<()>,
	shutdown_sender: oneshot::Sender<()>,
	runtime_handle: runtime::Handle,
}

impl RuntimeThread {
	pub fn start(runtime: Runtime) -> Self {
		let runtime_handle = runtime.handle().clone();
		let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();
		let join_handle = std::thread::spawn(move || {
			let _ = runtime.block_on(shutdown_receiver);
			const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
			runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
		});

		Self {
			join_handle,
			shutdown_sender,
			runtime_handle,
		}
	}

	pub fn handle(&self) -> &runtime::Handle {
		&self.runtime_handle
	}

	pub fn shut_down(self) -> anyhow::Result<()> {
		let _ = self.shutdown_sender.send(());
		self.join_handle
			.join()
			.map_err(|_| anyhow!("Failed to join tokio runtime thread"))
	}
}
