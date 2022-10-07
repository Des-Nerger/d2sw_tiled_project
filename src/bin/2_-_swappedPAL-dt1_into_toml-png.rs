#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::{dt1, unbuffered_stdout},
	png::ColorType,
	std::io::{self, BufWriter, Read, Write},
};

fn main() {
	let buffer = &mut Vec::<u8>::new();
	io::stdin().read_to_end(buffer).unwrap();
	let (swappedPAL, dt1) = {
		const PAL_LEN: usize = 256 * 3;
		buffer.as_slice().split_at(PAL_LEN)
	};
	#[allow(unused_variables)]
	let buffer = ();

	let dt1Metadata = &dt1::Metadata::new(dt1);
	let image = Image::new(&dt1Metadata.tiles);
	eprintln!("{:?}", (image.width, image.height));
	let mut stdout = BufWriter::new(unbuffered_stdout());
	let mut png = png::Encoder::new(&mut stdout, image.width as _, image.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPAL);
	png.write_header().unwrap().write_image_data(&image.data).unwrap();
	stdout.write_all(&(toml::to_vec(dt1Metadata).unwrap_or_else(|err| panic!("{err}")))).unwrap();

	struct Image {
		width: usize,
		height: usize,
		data: Vec<u8>,
	}
	impl Image {
		fn new(tiles: &[dt1::Tile]) -> Self {
			let width;
			let height = {
				const MAX_BLOCKHEIGHT: usize = 32;
				let mut y: usize = 0;
				for tile in tiles {
					const FLOOR: i32 = 0;
					const ROOF: i32 = 15;
					let blockHeight = if let FLOOR | ROOF = tile.orientation { 15 + 1 } else { MAX_BLOCKHEIGHT };
					y = y.nextMultipleOf(blockHeight) + tile.blocks.len() * blockHeight;
				}
				const BLOCKWIDTH: usize = 32;
				let requiredPixelArea = y * BLOCKWIDTH;
				width = ((requiredPixelArea as f32).sqrt() as usize).next_power_of_two();
				requiredPixelArea.divCeil(width).nextMultipleOf(MAX_BLOCKHEIGHT)
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

			Image { width, height, data: [98].repeat(height * width) }
		}
	}
	impl dt1::DrawDestination for Image {
		#[inline(always)]
		fn width(&self) -> usize {
			self.width
		}
		#[inline(always)]
		fn putpixel(&mut self, atIndex: usize, withValue: u8) {
			self.data[atIndex] = withValue;
		}
	}
}
