#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables)]

use {
	d2sw_tiled_project::{
		dt1::{self, DrawDestination},
		stdoutRaw, Image, PAL_LEN,
	},
	png::ColorType,
	std::io::{self, BufWriter, Read, Write},
};

fn main() -> Result<(), dt1::VersionMismatchError> {
	let buffer = &mut Vec::<u8>::new();
	io::stdin().read_to_end(buffer).unwrap();
	let (swappedPAL, dt1) = buffer.as_slice().split_at(PAL_LEN);
	#[allow(unused_variables)]
	let buffer = ();

	let dt1Metadata = &dt1::Metadata::new(dt1)?;
	let image = Image::fromDT1(&dt1Metadata.tiles, dt1);
	let stdout = &mut BufWriter::new(stdoutRaw());
	let toml = &toml::to_string(dt1Metadata).unwrap_or_else(|err| panic!("{err}"));
	write!(stdout, "{}\n{toml}", toml.len()).unwrap();
	let mut png = png::Encoder::new(stdout, image.width() as _, image.height() as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&image.data).unwrap();
	Ok(())
}
