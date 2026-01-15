pub struct World {
}

impl World {
	pub fn new() -> Self {
		Self {}
	}

	pub fn update(&mut self, _t_now: &std::time::Duration) -> Result<(), i32> {
		Ok(())
	}

	pub fn render(&self) -> Result<(), i32> {
		Ok(())
	}
}
