
use std::time::Duration;
use alto::Stereo;
use alto::Mono;
use std::time::Instant;
use lewton::VorbisError;
use lewton::inside_ogg::OggStreamReader;
use alto::Source;

pub struct AudioSys {
	alto: alto::Alto,
	device: Option<(alto::OutputDevice, alto::Context)>,
}

impl AudioSys {
	pub fn new() -> Self {
		let alto = alto::Alto::load_default().unwrap();
		let device = alto.open(None).ok()
			.and_then(|d| d.new_context(None).map(|ctx| (d, ctx)).ok());
		Self {
			device,
			alto,
		}
	}
	pub fn active_device(&self) -> Option<String> {
		use alto::DeviceObject;
		self.device.as_ref().and_then(|d| d.0.specifier()).map(|ds| ds.to_string_lossy().into_owned())
	}
	pub fn list_devices(&self) -> Vec<String> {
		self.alto.enumerate_outputs()
			.into_iter()
			.map(|cs| cs.to_string_lossy().into_owned())
			.collect()
	}
	pub fn play(&self, s: &mut Sound) {
		s.src.play();
	}
}

pub struct Sound {
	src: alto::StaticSource,
}

impl Sound {
	pub fn load(cxt: &alto::Context, p: &str) -> Self {
		let f = std::fs::File::open(p).expect("Can't open file");
		let mut srr = OggStreamReader::new(f).unwrap();
		let mut src = cxt.new_static_source()
			.expect("could not create streaming src");
		let sample_rate = srr.ident_hdr.audio_sample_rate as i32;

		if srr.ident_hdr.audio_channels > 2 {
			println!("Stream error: {} channels are too many!", srr.ident_hdr.audio_channels);
		}

		let sample_channels = srr.ident_hdr.audio_channels as f32 *
			srr.ident_hdr.audio_sample_rate as f32;

		let mut samples = vec![];
		let mut len = 0.;
		while let Some(pck_samples) = srr.read_dec_packet_itl().unwrap() {
			len += pck_samples.len() as f32 / sample_channels;
			samples.extend(pck_samples);
		}

		let buf = match srr.ident_hdr.audio_channels {
			1 => cxt.new_buffer::<Mono<i16>,_>(&samples, sample_rate),
			2 => cxt.new_buffer::<Stereo<i16>,_>(&samples, sample_rate),
			n => panic!("unsupported number of channels: {}", n),
		}.unwrap();

		src.set_buffer(buf.into());

		Self {
			src
		}

	}
}
