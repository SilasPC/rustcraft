
use std::time::Duration;
use alto::Stereo;
use alto::Mono;
use std::time::Instant;
use lewton::VorbisError;
use lewton::inside_ogg::OggStreamReader;
use alto::Source;

pub fn do_it() {
    let alto = alto::Alto::load_default().unwrap();

    for s in alto.enumerate_outputs() {
        println!("Found device: {}", s.to_str().unwrap());
    }
    
    let device = alto.open(None).unwrap(); // Opens the default audio device
    let cxt = device.new_context(None).unwrap(); // Creates a default context
    
    // Configure listener
    cxt.set_position([1.0, 4.0, 5.0]);
    cxt.set_velocity([2.5, 0.0, 0.0]);
    cxt.set_orientation(([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]));
    
    
    // Now you can load your samples and store them in a buffer with
    // `context.new_buffer(samples, frequency)`;

    let f = std::fs::File::open("assets/sample.ogg").expect("Can't open file");

	// Prepare the reading
	let mut srr = OggStreamReader::new(f).unwrap();

    let mut src = cxt.new_streaming_source()
		.expect("could not create streaming src");
	let sample_rate = srr.ident_hdr.audio_sample_rate as i32;

    if srr.ident_hdr.audio_channels > 2 {
		// the openal crate can't process these many channels directly
		println!("Stream error: {} channels are too many!", srr.ident_hdr.audio_channels);
	}

	println!("Sample rate: {}", srr.ident_hdr.audio_sample_rate);

	// Now the fun starts..
	let mut n = 0;
	let mut len_play = 0.0;
	let mut start_play_time = None;
	let start_decode_time = Instant::now();
	let sample_channels = srr.ident_hdr.audio_channels as f32 *
		srr.ident_hdr.audio_sample_rate as f32;
	while let Some(pck_samples) = srr.read_dec_packet_itl().unwrap() {
		println!("Decoded packet no {}, with {} samples.", n, pck_samples.len());
		n += 1;
		let buf = match srr.ident_hdr.audio_channels {
			1 => cxt.new_buffer::<Mono<i16>,_>(&pck_samples, sample_rate),
			2 => cxt.new_buffer::<Stereo<i16>,_>(&pck_samples, sample_rate),
			n => panic!("unsupported number of channels: {}", n),
		}.unwrap();

		src.queue_buffer(buf).unwrap();

		len_play += pck_samples.len() as f32 / sample_channels;
		// If we are faster than realtime, we can already start playing now.
		if n == 100 {
			let cur = Instant::now();
			if cur - start_decode_time < Duration::from_millis((len_play * 1000.0) as u64) {
				start_play_time = Some(cur);
				src.play();
			}
		}
	}
	let total_duration = Duration::from_millis((len_play * 1000.0) as u64);
	let sleep_duration = total_duration - match start_play_time {
			None => {
				src.play();
				Duration::from_millis(0)
			},
			Some(t) => (Instant::now() - t)
		};
	println!("The piece is {} s long.", len_play);
	std::thread::sleep(sleep_duration);
    panic!("done");

}