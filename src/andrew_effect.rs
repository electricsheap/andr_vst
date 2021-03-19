use std::{collections::{VecDeque}, iter::Filter, sync::{Arc, Weak}};
use crate::{AndrewParams, Conv};
use crate::biquad::{BiQuadraticFilter, FilterKind::{self, *}};

pub trait AndrewEffect {
	fn process( &mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32] );

	fn update_params( &mut self ) {}

	#[inline]
	fn get_params( &self ) -> Weak<AndrewParams> {
		Weak::new()
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
		// // out_buf.iter_mut().zip(in_buf.iter()).for_each(|(out, samp)| *out = 0.0);

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
}


pub struct TooSlewEffect {
	target_sample: [f32; 2],
	prev_sample: [f32; 2],
	amount: f32,
	params: Weak<AndrewParams>,
}

impl AndrewEffect for TooSlewEffect {
	fn process(&mut self, chan_id: usize, in_buf: &[f32], out_buf: &mut [f32]) {
		// // out_buf.iter_mut().zip(in_buf.iter()).for_each(|(out, samp)| *out = 0.0);

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

impl TooSlewEffect {
	pub fn new( params: Weak<AndrewParams> ) -> Self {
		TooSlewEffect {
			target_sample: [0.0; 2],
			prev_sample: [0.0; 2],
			amount: 1.0,
			params,
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
