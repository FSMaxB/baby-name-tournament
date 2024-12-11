use std::cell::OnceCell;
use std::ops::Deref;

pub struct ForceUnwrappedField<T>(OnceCell<T>);

impl<T> Default for ForceUnwrappedField<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> Deref for ForceUnwrappedField<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0.get().unwrap_or_else(|| {
			panic!(
				"Tried to read uninitialized field of type {}",
				std::any::type_name::<T>()
			)
		})
	}
}

impl<T> ForceUnwrappedField<T> {
	#[allow(unused)]
	pub fn new(value: T) -> Self {
		Self(value.into())
	}

	pub fn initialize(&self, value: T) {
		assert!(
			self.try_initialize(value).is_ok(),
			"Tried to initialize field of type {} that was already initialized",
			std::any::type_name::<T>()
		);
	}

	/// Returns the new value back if it was already initialized
	pub fn try_initialize(&self, value: T) -> Result<(), T> {
		self.0.set(value)
	}
}
