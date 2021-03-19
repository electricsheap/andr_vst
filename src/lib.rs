#![allow(unused_imports)]

#[macro_use]
extern crate vst;


use andrew_effect::*;
use vst::plugin::{Category, HostCallback, Info, Plugin, PluginParameters};
use vst::buffer::AudioBuffer;
use vst::util::AtomicFloat;


mod andrew_effect;
mod biquad;
use biquad::BiQuadraticFilter;


mod log;
use log::Logger;

mod types;


use std::{cell::{Ref, RefCell}, path::Path, rc::Weak, sync::atomic::{AtomicBool, Ordering}};
use std::sync::Arc;

#[derive(Default)]
struct Conv {
	sample_rate: f32,
	logger: Logger,
	params: Arc<AndrewParams>,
	effects: Vec<Box<dyn AndrewEffect>>,
}


impl Conv {

}

impl Plugin for Conv {

	fn new(_host: HostCallback) -> Self
	where Self: Sized + Default, {
		let params = Arc::new(<AndrewParams as Default>::default());


		let effects: Vec<Box<dyn AndrewEffect>> = vec![
			Box::new(FilterEffect::new(Arc::downgrade(&params))),
			Box::new(SlewEffect::new(Arc::downgrade(&params))),
		];

		Conv {
			sample_rate: 44100.0,
			logger: Logger::new( &Path::new("/Library/Audio/Plug-Ins/VST/Custom/conv_log.txt")),
			params,
			effects,
		}
	}

	fn get_info(&self) -> Info {
		Info {
			name: "Conv".into(),
			vendor: "Andrew Wilson".into(),
			inputs: 2,
			outputs: 2,
			parameters: 10,
			category: Category::Effect,
			..Default::default()
		}
	}
	
	fn set_sample_rate(&mut self, rate: f32) {
		self.sample_rate = rate;
		self.params.sample_rate.set(rate);
		self.logger.log(&format!("changed sample rate too {}", rate));
	}

	fn init( &mut self ) {}

	fn process( &mut self, buffer: &mut AudioBuffer<f32> ) {
		// must have at most two channels 
		if buffer.input_count() > 2 { return }

		//update params
		if self.params.updated.load( Ordering::Relaxed ) {
			self.logger.log("updating effect params");
			self.effects.iter_mut().for_each(|effect| effect.update_params() );
			self.params.updated.store( false, Ordering::Relaxed )
		}

		let buf_len = buffer.samples(); 
		// loop over AndrewEffects
		
		for (chan_id, (in_chan, out_chan)) in buffer.zip().enumerate() {
			// out_chan.iter_mut().zip(in_chan.iter()).for_each(|(out, samp)| *out = *samp);
			let in_vec = Vec::from(in_chan);
			let out_vec = vec![0f32; buf_len];

			let mut buf = (in_vec, out_vec);
			for effect in self.effects.iter_mut() {
				// flips the bufs beforehand so that an extra flip
				// is not needed after the loop has finished
				buf = (buf.1, buf.0);

				// consequentially, the bufs are given to process in the opposite order
				effect.process(chan_id, &buf.1, &mut buf.0);
			}

			let dry_wet = self.params.dry_wet.get();
			for i in 0..buf.0.len() {
				out_chan[i] = (buf.0[i] * dry_wet)  +  (in_chan[i] * (1.0 - dry_wet));
			}
		}
			
	} 

	fn get_parameter_object( &mut self ) -> Arc<dyn PluginParameters> {
		Arc::clone( &self.params ) as Arc<dyn PluginParameters>
	}
}


pub struct AndrewParams {
	updated: AtomicBool,
	sample_rate: AtomicFloat,
	dry_wet: AtomicFloat,
	slew: AtomicFloat,
	delay_time: AtomicFloat,
	delay_feedback: AtomicFloat,

	// filter
	cutoff: AtomicFloat,
}

impl PluginParameters for AndrewParams {
	fn get_parameter( &self, i: i32 ) -> f32 {
		match i {
			0 => self.dry_wet.get(),
			1 => self.slew.get(),
			2 => self.delay_time.get(),
			3 => self.delay_feedback.get(),
			4 => self.cutoff.get(),
			_ => 0.0,
		}
	}

	fn get_parameter_label(&self, i: i32) -> String {
		match i {
			1 => "Hz",
			_ => "",
		}.into()
	}

	fn get_parameter_text( &self, i: i32 ) -> String {
		match i {
			0 => format!("{:.1}", self.dry_wet.get()).into(),
			1 => format!("{:.1}", self.slew.get() / 100.0).into(),
			2 => format!("{:.1}", self.delay_time.get() * 100.0).into(),
			3 => format!("{:.1}", self.delay_feedback.get() * 100.0).into(),
			4 => format!("{:.1}", self.cutoff.get()).into(),
			_ => "0.0".into(),
		}
	}

	fn get_parameter_name( &self, i: i32 ) -> String {
		match i {
			0 => "dry_wet",
			1 => "slew",
			2 => "delay_time",
			3 => "delay_feedback",
			4 => "cutoff",
			_ => "",
		}.into()
	}

	fn set_parameter(&self, i: i32, val: f32) {
        match i {
			0 => self.dry_wet.set(val),
			1 => self.slew.set(20_f32 * 1000_f32.powf(val)),
			2 => self.delay_time.set(1.0 - val * 0.99),
			3 => self.delay_feedback.set(val),
			4 => self.cutoff.set(val * 0.99 + 0.01),
            _ => (),
        }
		self.updated.store(true, Ordering::Relaxed);
    }
}

impl Default for AndrewParams {
	fn default() -> Self {
		AndrewParams {
			updated: AtomicBool::new(true),
			sample_rate: AtomicFloat::new(44100.0),
			dry_wet: AtomicFloat::new(1.0),
			slew: AtomicFloat::new(1.0),
			delay_time: AtomicFloat::new(0.01),
			delay_feedback: AtomicFloat::new(1.0),

			cutoff: AtomicFloat::new(1.0),
		}
	}
}

plugin_main!(Conv); // Important!



// #[cfg(test)]
// mod tests {
// 	use super::*;

// }
