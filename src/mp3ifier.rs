
use lame::Lame;
use puremp3::{*, self, FrameHeader, Mp3Decoder, decoder::process_frame};


use std::{default, io::Read};
use std::sync::{Arc, Condvar, Mutex, mpsc::channel};

pub struct Mp3ifier<'a> {
	input_buffer: Rc<(Vec<f32>, Vec<f32>)>,
	mp3_buffer: Vec<u8>,
	output_buffer: Rc<(Vec<f32>, Vec<f32>)>,
	lame: Option<Lame>,
	decoder: Mp3Decoder<&'a [u8]>,
	format: FrameHeader,
}

impl<'a> Mp3ifier<'a> {
	fn new( kbs: BitRate,  ) -> Self {
		let mut lame = Lame::new();
		let mut kbs = kbs;
		// let format = FrameHeader {
		// 	bitrate: Kbps24,
		// 	sample_rate: SampleRate::Hz44100,
		// 	layer: MpegLayer::Layer3,
		// 	original: true,
		// 	crc: false,
		// 	padding: false,
		// 	channels: Channels::Stereo,
		// 	copyright: false,
		// 	version: MpegVersion::Mpeg2,
		// 	emphasis: Emphasis::None,
		// 	data_size: 0,
		// 	sample_rate_table: 0,
		// };
		if let Some(lame) = lame.as_mut() {
			lame.init_params();
			lame.set_quality(9);
			lame.set_kilobitrate(16);
			lame.set_sample_rate(/*self.sample_rate*/44100);
		}

		
		Mp3ifier {
			input_buffer: (vec![], vec![]), 
			mp3_buffer: (vec![]),
			output_buffer: (vec![], vec![]), 
			lame,
			decoder,
			format,
		}
	}
}