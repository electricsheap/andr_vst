
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;


#[derive(Default)]
pub struct Logger {
	file: Option<Box<File>>
}

impl Logger {

	pub fn new( path: &Path ) -> Self {
		let file = match OpenOptions::new().append( true ).open( path ) {
			Ok( file ) => Some( Box::new(file) ),
			Err( _ ) => match File::create( path ) {
				Ok( file ) => Some( Box::new(file) ),
				Err( _ ) => None
			},
		};
		Logger { file }
	}

	fn log_inner( &mut self, text: &str ) -> Result<(), Box<dyn Error>> {
		match &mut self.file {
			Some( file ) => {
				file.write( text.as_bytes() )?;
				file.write( b"\r\n" )?;
				file.flush()?;
				Ok(())
			}
			None => Ok(()),
		}
	}
	
	pub fn log( &mut self, text: &str ) {
		if let Err( why ) = self.log_inner( text ) {
			eprintln!( "Conv Plugin Error: {}", why );
		}
	}
}