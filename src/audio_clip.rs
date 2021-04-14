

#[derive(Clone)]
pub struct AudioClip {
	buf: Vec<f32>,
	pub scale: f32,
	playhead: f32,
	base_sample_rate: f32,
	looping: bool,
}

impl AudioClip {


	pub fn new( base_size: usize, scale: f32, base_sample_rate: f32, looping: bool ) -> Self {
		AudioClip {
			buf: vec![0.0; base_size],
			scale,
			base_sample_rate,
			playhead: 0.0,
			looping: false,
		}
	}

	pub fn grain( buf: &[f32], scale: f32, base_sample_rate: f32 ) -> Self {
		AudioClip {
			buf: Vec::from(buf),
			scale,
			base_sample_rate,
			playhead: 0.0,
			looping: true,
		}
	}

	#[inline]
	pub fn extend( &mut self, val: &[f32] ) {
		self.buf.extend_from_slice( val );
	}

	#[inline]
	pub fn read_playhead( &mut self ) -> f32 {
		let val = self.interp(self.playhead);
		self.playhead += self.scale;
		val
	}


	// shrinks the vec and drops values which the playhead has completely passed
	// drops until index playhead.floor() 
	// must be a buffer of as least 4 extra samples to be safe
	pub fn shrink( &mut self ) {
		if self.playhead > 1.0 {
			let max = self.playhead.floor() as usize - 1;
			self.buf.drain(0..max);
			self.playhead -= max as f32;
		}
	}

	pub fn interp( &self, i: f32 ) -> f32 {
		if i + 2.0 > self.buf.len() as f32 {
			0.0
		} else {
			let i_floor = i.floor() as usize;
			let rem = i % 1.0;
			let val = self.buf[i_floor] * (1.0 - rem) + self.buf[i_floor + 1] * rem;
			val
		}
	}
}

pub struct Grain {
	buf: Vec<f32>,
	scale: f32,
	playhead: f32,
}

impl Grain {

}
