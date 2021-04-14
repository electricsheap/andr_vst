
use std::{f32::consts::PI, time::Instant};
const TAU : f32 = PI * 2.0;

trait Modulator {
	fn start( &mut self );
	fn stop( &mut self );
	fn get( &self ) -> f32;
}


pub struct Lfo { 
	cached: f32,
	time: u32,
}

impl Lfo {
	pub fn get( &self ) -> f32 {
		(( self.time as f32 / 44100.0) as f32 * TAU ).sin()
	}

	pub fn forward( &mut self, time: u32 ) {
		self.time += time;
		self.time %= 44100;
	}
}

impl Default for Lfo {
	fn default() -> Self {
		Lfo {
			cached: 0.0,
			time: 0,
		}
	}
}

