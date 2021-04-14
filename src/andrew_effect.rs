use std::{collections::{VecDeque}, iter::Filter, ops::Mul, sync::{Arc, Weak}, f32::consts};
use crate::{AndrewParams, AndrewVst, audio_clip::AudioClip, modulator::Lfo};
use crate::biquad::{BiQuadraticFilter, FilterKind::{self, *}};

pub trait AndrewEffect {
	fn process( &mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32] ) {
		(0..in_buf.len()).for_each(|i| out_buf[i] = in_buf[i] );
	}

	fn update_params( &mut self ) {}

	fn get_latency( &self ) -> usize {1}
}

#[derive(Default)]
pub struct DistEffect {
	params: Weak<AndrewParams>,
	gain: f32,
}


impl AndrewEffect for DistEffect {
	fn process(&mut self, _chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {

		let d = self.gain;
		let n = ((1.0 +  4.0/d).sqrt() - 1.0) * 0.5;

		for i in 0..in_buf.len() {

			let x = out_buf[i] * self.gain;
			let y = 1.0 - 1.0/(d*(x+n)) + n;
			out_buf[i] = y;
		}
	}

	fn update_params(&mut self) {
		if let Some(params) = self.params.upgrade() {
			self.gain = params.delay_feedback.get() * 2.0;
		}
	}
}

impl DistEffect {
	pub fn new( params: Weak<AndrewParams> ) -> Self {
		DistEffect {
			params,
			gain: 1.0,
		}
	}
}




pub struct GrainShiftEffect {
	grains: [Vec<AudioClip>; 2],
	pitch: f32,
}

impl AndrewEffect for GrainShiftEffect {

}



pub struct VibEffect {
	bufs: [AudioClip; 2],
	lfo: [Lfo; 2],
}

impl AndrewEffect for VibEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
		if chan_id > 2 { return }
		self.lfo[chan_id].forward( in_buf.len() as u32 );
		self.bufs[chan_id].scale = 1.0 - (self.lfo[chan_id].get() * 0.02);
		self.bufs[chan_id].extend( &in_buf );

		out_buf.iter_mut().for_each(|out| *out = self.bufs[chan_id].read_playhead() );
		self.bufs[chan_id].shrink();
	}
}


impl VibEffect {
	pub fn new() -> Self {
		let buf = AudioClip::new(512, 1.0, 44100.0, false);
		VibEffect {
			bufs: [buf.clone(), buf.clone()],
			lfo: [Lfo::default(), Lfo::default()],
		}
	}
}

pub struct FilterEffect {
	state: [BiQuadraticFilter; 2],
	params: Weak<AndrewParams>,
}


impl AndrewEffect for FilterEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32] ) {
		in_buf.iter()
		.zip(out_buf.iter_mut())
		.for_each(|(samp, out)| *out = self.state[chan_id].filter(*samp));
	}

	fn update_params(&mut self) {
		if let Some(params) = self.params.upgrade() {
			for biquad in self.state.iter_mut() {
				biquad.update_center_freq(params.cutoff.get() * 10_000.0);
			}
		}
	}
}

impl FilterEffect {
	pub fn new(params: Weak<AndrewParams>) -> Self {
		let x = BiQuadraticFilter::new(CUSTOM, 1000.0, 44100.0, 1.0, 0.0);
		FilterEffect {
			state: [x.clone(), x.clone()],
			params,
		}
	}
}




pub struct ConvEffect {
	buf: [VecDeque<f32>; 2],
	pattern: Vec<f32>,
	spread: f32,
	buf_len: usize,

	params: Weak<AndrewParams>
}

impl AndrewEffect for ConvEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {

		self.buf[chan_id].extend(in_buf);
		out_buf.iter_mut()
		.for_each(|out| {
			self.buf[chan_id].pop_front();
			*out = self.pattern.iter()
				.zip(self.buf[chan_id].iter())
				.map(|elm| elm.0 * elm.1 * (1.5/self.spread))
				.sum::<f32>();
		});
	}

	fn update_params(&mut self) {
		if let Some(params) = self.params.upgrade() {
			let spread = params.delay_time.get() * 100.0;
			let len = self.buf_len.clone();
			if spread != self.spread {

				self.spread = spread;
				self.pattern = (0..len)
					.map(|elm| elm - (len/2))
					.map(|elm| elm as f32)
					.map(|elm| 1.0 - (elm/spread).powi(2).min(1.0))
					.collect();
			}
		}
	}

}

impl ConvEffect {
	pub fn new(params: Weak<AndrewParams>) -> Self {
		let len = 100;//pattern.len();
		let spread = 6.0;

		let pattern = (0..len)
			.map(|elm| elm - (len/2))
			.map(|elm| elm as f32)
			.map(|elm| 1.0 - (elm/spread).powi(2).min(1.0))
			.collect();
		// let dc = pattern.iter().sum::<f32>();
		// let pattern: Vec<f32> = pattern.iter().map(move |elm| *elm/dc).collect();
		ConvEffect {
			buf: [VecDeque::from(vec![0.0; len]), VecDeque::from(vec![0.0; len])],
			buf_len: len,
			pattern,
			params,
			spread: 6.0,
		}
	}
}




pub struct PrimeEffect {
	prev_sample: [f32; 2],
}

impl AndrewEffect for PrimeEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
		in_buf.iter()
		.zip(out_buf.iter_mut())
		.for_each(|(sample, out)| {
			*out = *sample - self.prev_sample[chan_id];
			self.prev_sample[chan_id] = *sample
		});
	}
}

impl PrimeEffect {
	pub fn new(params: Weak<AndrewParams>) -> Self {
		PrimeEffect {
			prev_sample: [0.0; 2],
		}
	}
}




pub struct IntEffect {
	sum: [f32; 2],
	avg_buf: [VecDeque<f32>; 2],
	buf_len: usize,
}

impl AndrewEffect for IntEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
		in_buf.iter()
		.zip(out_buf.iter_mut())
		.for_each(|(sample, out)| {
			let avg = self.avg_buf[chan_id].iter().sum::<f32>()/(self.buf_len as f32);
			
			self.sum[chan_id] += *sample;
			self.sum[chan_id] -= avg * (20.0 / 44100.0);
			
			*out = self.sum[chan_id] - avg * (1.0 - (20.0 / 44100.0));
			self.avg_buf[chan_id].push_front( self.sum[chan_id] );
			self.avg_buf[chan_id].pop_back();
		});
	}

}

impl IntEffect {
	pub fn new(params: Weak<AndrewParams>) -> Self {
		IntEffect {
			sum: [0.0; 2],
			avg_buf: [VecDeque::from(vec![0.0; 1000]), VecDeque::from(vec![0.0; 1000])],
			buf_len: 1000,
		}
	}
}




pub struct SlewEffect {
	target_sample: [f32; 2],
	prev_sample: [f32; 2],
	amount: f32,
	params: Weak<AndrewParams>,
}

impl AndrewEffect for SlewEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
	
		let max_b = self.amount;

		for i in 0..in_buf.len() {
			self.target_sample[chan_id] = in_buf[i];
			let d = self.target_sample[chan_id] - self.prev_sample[chan_id];
			let clamped = d.max( -max_b ).min( max_b );
			out_buf[i] = self.prev_sample[chan_id] + clamped;
			self.prev_sample[chan_id] = out_buf[i];
		}
	}

	fn update_params(&mut self) {
		if let Some(params) = self.params.upgrade() {
			self.amount = params.slew.get() / ( params.sample_rate.get() );
		}
	}
}

impl SlewEffect {
	pub fn new( params: Weak<AndrewParams> ) -> Self {
		SlewEffect {
			target_sample: [0.0; 2],
			prev_sample: [0.0; 2],
			amount: 1.0,
			params,
		}
	}

	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
	
		let max_b = self.amount;

		for i in 0..in_buf.len() {
			self.target_sample[chan_id] = in_buf[i];
			let d = self.target_sample[chan_id] - self.prev_sample[chan_id];
			let clamped = d.max( -max_b ).min( max_b );
			out_buf[i] = self.prev_sample[chan_id] + clamped;
			self.prev_sample[chan_id] = out_buf[i];
		}
	}

	fn update_params(&mut self) {
		if let Some(params) = self.params.upgrade() {
			self.amount = params.slew.get() / ( params.sample_rate.get() );
		}
	}
}


pub struct TooSlewEffect {
	target_sample: [f32; 2],
	prev_sample: [f32; 2],

	target_slope: [f32; 2],
	prev_slope: [f32; 2],
	amount: f32,

	params: Weak<AndrewParams>,
}

impl TooSlewEffect {
	pub fn new( params: Weak<AndrewParams> ) -> Self {
		TooSlewEffect {
			target_sample: [0.0; 2],
			prev_sample: [0.0; 2],
			target_slope: [0.0; 2],
			prev_slope: [0.0; 2],
			amount: 1.0,
			params,
		}
	}
}

impl AndrewEffect for TooSlewEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
		// // out_buf.iter_mut().zip(in_buf.iter()).for_each(|(out, samp)| *out = 0.0);

		let max_b = self.amount;

		for i in 0..in_buf.len() {
			self.target_sample[chan_id] = in_buf[i];
			self.target_slope[chan_id] 	= self.target_sample[chan_id] - self.prev_sample[chan_id];
			let accel  					= self.target_slope[chan_id] - self.prev_slope[chan_id];

			let slope 					= self.prev_slope[chan_id] + accel;
			let sample 					= self.prev_sample[chan_id] + slope;

			out_buf[i] = sample;
			self.prev_sample[chan_id] = sample;
			self.prev_slope[chan_id] = slope;
		}
	}

	fn update_params(&mut self) {
		if let Some(params) = self.params.upgrade() {
			self.amount = params.slew.get() / ( params.sample_rate.get() );
		}
	}
}




pub struct DelayEffect {
	buffer: Vec<VecDeque<f32>>,
}

impl AndrewEffect for DelayEffect {
	#[allow(unused)]
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
			for i in 0..out_buf.len() {
			let delayed = self.buffer[chan_id].pop_front().unwrap_or(0.0);
			self.buffer[chan_id].push_back(delayed * 0.5);
			out_buf[i] = in_buf[i] + delayed;
		}
	}
}

impl Default for DelayEffect {
	fn default() -> Self {
		DelayEffect {
			buffer: vec![VecDeque::from(vec![0.0; 44100]),VecDeque::from(vec![0.0; 44100])],
		}
	}
}
