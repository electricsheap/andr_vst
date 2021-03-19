
const TAU: f32 = 2.0 * std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum FilterKind {
	LOWPASS,
	HIGHPASS,
	BANDPASS,
	NOTCH,
	PEAK,
	LOWSHELF,
	HIGHSHELF,
	CUSTOM,
}

use FilterKind::*;

impl Default for FilterKind {
	fn default() -> Self {
		FilterKind::LOWPASS
	}
}

#[derive(Default, Clone)]
pub struct BiQuadraticFilter {
	kind: FilterKind,
	a0: f32, 
	a1: f32, 
	a2: f32, 
	b0: f32, 
	b1: f32, 
	b2: f32,
	x1: f32, 
	x2: f32, 
	y: f32, 
	y1: f32, 
	y2: f32,
	gain_abs: f32,
	center_freq: f32, 
	sample_rate: f32, 
	q: f32, 
	gain_db: f32,
}

impl BiQuadraticFilter {

	pub fn new(kind: FilterKind, center_freq: f32, sample_rate: f32, q: f32, gain_db: f32) -> Self {
		let mut ret = BiQuadraticFilter::default();
		ret.recfg(kind, center_freq, sample_rate, q, gain_db);
		ret
	}

	pub fn update_center_freq( &mut self, center_freq: f32 ) {
		if self.center_freq != center_freq {
			self.recfg(self.kind, center_freq, self.sample_rate, self.q, self.gain_db);
		}
	}

	pub fn recfg(&mut self, kind: FilterKind, center_freq: f32, sample_rate: f32, q: f32, gain_db: f32) {
		let a0;
		let mut a1;
		let mut a2;
		let mut b0;
		let mut b1;
		let mut b2;
		let x1 = self.x1;//0f32; 
		let x2 = self.x2;//0f32; 
		let y1 = self.y1;//0f32;
		let y2 = self.y2;//0f32;
		let q = if q == 0.0 { 1e-9 } else { q };
		let cf = center_freq;

		// only used for peaking and shelving filters
		let gain_abs = (10f32).powf(gain_db / 40.0);
		let omega = TAU * cf / sample_rate;
		let sn = omega.sin();
		let cs = omega.cos();
		let alpha = sn / (2.0 * q);
		let beta = (gain_abs + gain_abs).sqrt();
		match kind {
			BANDPASS => {
				b0 = alpha;
				b1 = 0.0;
				b2 = -alpha;
				a0 = 1.0 + alpha;
				a1 = -2.0 * cs;
				a2 = 1.0 - alpha;
			},
			LOWPASS => {
				b0 = (1.0 - cs) / 2.0;
				b1 = 1.0 - cs;
				b2 = (1.0 - cs) / 2.0;
				a0 = 1.0 + alpha;
				a1 = -2.0 * cs;
				a2 = 1.0 - alpha;
			},
			HIGHPASS => {	
				b0 = (1.0 + cs) / 2.0;
				b1 = -(1.0 + cs);
				b2 = (1.0 + cs) / 2.0;
				a0 = 1.0 + alpha;
				a1 = -2.0 * cs;
				a2 = 1.0 - alpha;
			},
			NOTCH => {	
				b0 = 1.0;
				b1 = -2.0 * cs;
				b2 = 1.0;
				a0 = 1.0 + alpha;
				a1 = -2.0 * cs;
				a2 = 1.0 - alpha;
			},
			PEAK => {	
				b0 = 1.0 + (alpha * gain_abs);
				b1 = -2.0 * cs;
				b2 = 1.0 - (alpha * gain_abs);
				a0 = 1.0 + (alpha / gain_abs);
				a1 = -2.0 * cs;
				a2 = 1.0 - (alpha / gain_abs);
			},
			LOWSHELF => {	
				b0 = gain_abs * ((gain_abs + 1.0) - (gain_abs - 1.0) * cs + beta * sn);
				b1 = 2.0 * gain_abs * ((gain_abs - 1.0) - (gain_abs + 1.0) * cs);
				b2 = gain_abs * ((gain_abs + 1.0) - (gain_abs - 1.0) * cs - beta * sn);
				a0 = (gain_abs + 1.0) + (gain_abs - 1.0) * cs + beta * sn;
				a1 = -2.0 * ((gain_abs - 1.0) + (gain_abs + 1.0) * cs);
				a2 = (gain_abs + 1.0) + (gain_abs - 1.0) * cs - beta * sn;
			},
			HIGHSHELF => {	
				b0 = gain_abs * ((gain_abs + 1.0) + (gain_abs - 1.0) * cs + beta * sn);
				b1 = -2.0 * gain_abs * ((gain_abs - 1.0) + (gain_abs + 1.0) * cs);
				b2 = gain_abs * ((gain_abs + 1.0) + (gain_abs - 1.0) * cs - beta * sn);
				a0 = (gain_abs + 1.0) - (gain_abs - 1.0) * cs + beta * sn;
				a1 = 2.0 * ((gain_abs - 1.0) - (gain_abs + 1.0) * cs);
				a2 = (gain_abs + 1.0) - (gain_abs - 1.0) * cs - beta * sn;
			},
			CUSTOM => {
				b0 = -alpha;
				b1 = 0.0;
				b2 = alpha;
				a0 = 2.0 + alpha;
				a1 = -2.0 * cs;
				a2 = 2.0 - alpha;
			}
		}

		b0 /= a0;
		b1 /= a0;
		b2 /= a0;
		a1 /= a0;
		a2 /= a0;

		self.kind = kind;
		self.a0 = a0; 
		self.a1 = a1; 
		self.a2 = a2; 
		self.b0 = b0; 
		self.b1 = b1; 
		self.b2 = b2;
		self.x1 = x1; 
		self.x2 = x2; 
		self.y1 = y1; 
		self.y2 = y2;
		self.gain_abs = gain_abs;
		self.center_freq = center_freq; 
		self.sample_rate = sample_rate; 
		self.q = q; 
		self.gain_db = gain_db;

	}


	// // provide a static amplitude result for testing
	// fn result(&self, f: f32) -> f32 {
	// 	let phi = (TAU * f / (2.0 * self.sample_rate)).sin().powi(2);
	// 	let r = ((b0 + b1 + b2, 2.0) - 4.0 * (b0 * b1 + 4.0 * b0 * b2 + b1 * b2) * phi + 16.0 * b0 * b2 * phi * phi) / ((1.0 + a1 + a2).powi(2) - 4.0 * (a1 + 4.0 * a2 + a1 * a2) * phi + 16.0 * a2 * phi * phi);
	// 	if ( r < 0.0 ) {
	// 		r = 0.0;
	// 	}
	// 	return r.sqrt();
	// }

	// // provide a static decibel result for testing
	// fn log_result(&self, f: f32) -> f32 {
	// 	let r: f32;
	// 	r = 20 * self.result(f).log10();
	// 	if ( r.is_infinite() || r.is_nan() )
	// 	{ r = -100; }
	// 	return r;
	// }

	// return the constant set for this filter
	#[allow(unused)]
	pub fn constants(&self) -> [f32; 5] {
		[self.a1, self.a2, self.b0, self.b1, self.b2]
	}
	// perform one filtering step
	pub fn filter(&mut self, x: f32) -> f32 {
			self.y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
			self.x2 = self.x1;
			self.x1 = x;
			self.y2 = self.y1;
			self.y1 = self.y;
			self.y
	}
}





