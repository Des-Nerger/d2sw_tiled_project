#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::dt1,
	png::ColorType,
	std::io::{self, BufWriter, Read},
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
	let imageData = ImageData::new(&dt1Metadata.tiles);
	eprintln!("{:?}", (imageData.width, imageData.height));
	let mut png = png::Encoder::new(
		// FIXME: remove this extra BufWriter layer when they fix http://github.com/rust-lang/rust/issues/60673
		BufWriter::new(io::stdout()),
		imageData.width as _,
		imageData.height as _,
	);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPal);
	png.write_header().unwrap().write_image_data(imageData.bytes.as_slice()).unwrap();

	struct ImageData {
		width: usize,
		height: usize,
		bytes: Vec<u8>,
	}
	impl ImageData {
		fn new(tiles: &[dt1::Tile]) -> Self {
			let width;
			let height = {
				let mut y: usize = 0;
				for tile in tiles {
					const FLOOR: i32 = 0;
					const ROOF: i32 = 15;
					let blockHeight = if let FLOOR | ROOF = tile.orientation { 15 + 1 } else { 32 };
					y = y.nextMultipleOf(blockHeight);
					y += tile.blocks.len() * blockHeight;
				}
				let requiredPixelArea = y * 32;
				width = ((requiredPixelArea as f32).sqrt() as usize).next_power_of_two();
				requiredPixelArea.divCeil(width).nextMultipleOf(32)
			};

			trait IntRoundings {
				fn nextMultipleOf(self, rhs: Self) -> Self;
				fn divCeil(self, rhs: Self) -> Self;
			}
			impl IntRoundings for usize {
				#[inline(always)]
				fn nextMultipleOf(self, rhs: Self) -> Self {
					self
						+ match self % rhs {
							0 => 0,
							r => (rhs - r),
						}
				}
				#[inline(always)]
				fn divCeil(self, rhs: Self) -> Self {
					let d = self / rhs;
					let r = self % rhs;
					d + if r > 0 && rhs > 0 { 1 } else { 0 }
				}
			}

			ImageData { width, height, bytes: [98].repeat(height * width) }
		}
	}
	impl dt1::DrawDestination for ImageData {
		#[inline(always)]
		fn width(&self) -> usize {
			self.width
		}
		#[inline(always)]
		fn putpixel(&mut self, atIndex: usize, withValue: u8) {
			self.bytes[atIndex] = withValue;
		}
	}
}
