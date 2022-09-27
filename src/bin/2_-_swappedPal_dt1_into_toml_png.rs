#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::dt1,
	png::ColorType,
	std::io::{self, Read},
};

fn main() {
	let buffer = &mut Vec::<u8>::new();
	io::stdin().read_to_end(buffer).unwrap();
	let (swappedPal, dt1) = {
		const PAL_LEN: usize = 256 * 3;
		buffer.as_slice().split_at(PAL_LEN)
	};
	#[allow(unused_variables)]
	let buffer = ();

	let dt1Metadata = &dt1::Metadata::new(dt1);
	let dt1TOML = toml::to_string(dt1Metadata).unwrap_or_else(|err| panic!("{err}"));
	print!("{}\n{dt1TOML}", dt1TOML.len());
	// for {}
	let mut png = png::Encoder::new(
		// I hope having LineWriter instead of BufWriter won't harm png::Encoder performance much
		io::stdout(),
		256,
		256,
	);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPal);
	// png.write_header().unwrap().write_image_data(&imageData).unwrap();
}
