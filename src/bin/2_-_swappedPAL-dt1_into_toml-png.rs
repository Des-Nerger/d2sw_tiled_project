#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::{
		dt1::{self, DrawDestination},
		unbuffered_stdout, PAL_LEN,
	},
	png::ColorType,
	std::io::{self, BufWriter, Read, Write},
};

fn main() {
	let buffer = &mut Vec::<u8>::new();
	io::stdin().read_to_end(buffer).unwrap();
	let (swappedPAL, dt1) = buffer.as_slice().split_at(PAL_LEN);
	#[allow(unused_variables)]
	let buffer = ();

	let dt1Metadata = &dt1::Metadata::new(dt1);
	let image = Image::new(&dt1Metadata.tiles, dt1);
	eprintln!("{:?}", (image.width(), image.height));
	let stdout = &mut BufWriter::new(unbuffered_stdout());
	let toml = &toml::to_string(dt1Metadata).unwrap_or_else(|err| panic!("{err}"));
	write!(stdout, "{}\n{toml}", toml.len()).unwrap();
	let mut png = png::Encoder::new(stdout, image.width() as _, image.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPAL);
	png.write_header().unwrap().write_image_data(&image.data).unwrap();

	struct Image {
		widthLog2: usize,
		height: usize,
		data: Vec<u8>,
	}
	impl Image {
		fn new(tiles: &[dt1::Tile], dt1: &[u8]) -> Self {
			trait NonZeroInteger {
				fn nextShlOf(self, rhs: Self) -> Self;
				fn shrCeil(self, rhs: Self) -> Self;
			}
			impl NonZeroInteger for usize {
				#[inline(always)]
				fn nextShlOf(self, rhs: Self) -> Self {
					let rhsExp2 = 1 << rhs;
					let r = self & (rhsExp2 - 1);
					self + ((!((r != 0) as Self) + 1) & (rhsExp2 - r))
				}
				#[inline(always)]
				fn shrCeil(self, rhs: Self) -> Self {
					((self - 1) >> rhs) + 1
				}
			}
			#[inline(always)]
			const fn log2(of: usize) -> usize {
				(usize::BITS - 1 - of.leading_zeros()) as _
			}
			macro_rules! log2 {
				( $of:expr ) => {{
					const LOG2: usize = log2($of);
					LOG2
				}};
			}

			const BLOCKWIDTH: usize = 32;
			const FLOOR_ROOF_BLOCKHEIGHT: usize = 15 + 1;
			const MAX_BLOCKHEIGHT: usize = 32;
			const FLOOR: i32 = 0;
			const ROOF: i32 = 15;
			let widthLog2;
			let height = {
				let mut y = 0;
				for tile in tiles {
					let blockHeightLog2 = if matches!(tile.orientation, FLOOR | ROOF) {
						log2!(FLOOR_ROOF_BLOCKHEIGHT)
					} else {
						log2!(MAX_BLOCKHEIGHT)
					};
					y = y.nextShlOf(blockHeightLog2) + (tile.blocks.len() << blockHeightLog2);
				}
				let requiredPixelArea = y << log2!(BLOCKWIDTH);
				widthLog2 = log2(((requiredPixelArea as f32).sqrt() as usize).next_power_of_two());
				requiredPixelArea.shrCeil(widthLog2).nextShlOf(log2!(FLOOR_ROOF_BLOCKHEIGHT))
			};
			let mut image = Self { widthLog2, height, data: [98].repeat(height << widthLog2) };
			let (mut x, mut y) = (0, 0);
			for tile in tiles {
				let blockHeight = {
					let blockHeightLog2 = if matches!(tile.orientation, FLOOR | ROOF) {
						log2!(FLOOR_ROOF_BLOCKHEIGHT)
					} else {
						log2!(MAX_BLOCKHEIGHT)
					};
					y = y.nextShlOf(blockHeightLog2);
					1 << blockHeightLog2
				};

				for block in &tile.blocks {
					let nextY = {
						let nextY = y + blockHeight;
						if nextY > height {
							x += BLOCKWIDTH;
							y = 0;
							blockHeight
						} else {
							nextY
						}
					};
					(if block.format == 1 { Self::drawBlockIsometric } else { Self::drawBlockNormal })(
						&mut image,
						x,
						y,
						&dt1[(tile.blockHeadersPointer + block.fileOffset) as _..][..block.length as _],
					);
					y = nextY;
				}
			}
			image
		}
	}
	impl DrawDestination for Image {
		#[inline(always)]
		fn widthLog2(&self) -> usize {
			self.widthLog2
		}
		#[inline(always)]
		fn putpixel(&mut self, atIndex: usize, withValue: u8) {
			self.data[atIndex] = withValue;
		}
	}
}
