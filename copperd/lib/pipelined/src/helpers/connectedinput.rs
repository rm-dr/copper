#[derive(Clone)]
pub enum ConnectedInput<T> {
	/// This input hasn't been connected yet
	NotConnected,

	/// This input has been connected, but hasn't been set
	Connected,

	/// This input has been connected and set
	Set { value: T },
}

impl<'a, T> ConnectedInput<T> {
	pub fn is_connected(&self) -> bool {
		matches!(self, Self::Connected | Self::Set { .. })
	}

	pub fn is_set(&self) -> bool {
		matches!(self, Self::Set { .. })
	}

	pub fn connect(&mut self) {
		if self.is_connected() {
			unreachable!("connected to an input twice")
		}

		*self = Self::Connected
	}

	pub fn set(&mut self, value: T) {
		if matches!(self, Self::NotConnected) {
			unreachable!("tried to set a disconnected input")
		}

		*self = Self::Set { value }
	}

	pub fn value(&'a self) -> Option<&'a T> {
		return match self {
			Self::Set { value } => Some(&value),
			_ => None,
		};
	}
}
