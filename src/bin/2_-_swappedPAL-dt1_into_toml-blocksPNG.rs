#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	core::mem::{transmute, MaybeUninit},
	d2sw_tiled_project::{
		array_fromFn,
		dt1::{self, DrawDestination},
		log2, stdoutRaw, NonZeroIntegerExt, TileColumns, PAL_LEN,
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
	eprintln!("{:?}", (image.width(), image.height));
	let stdout = &mut BufWriter::new(stdoutRaw());
	let toml = &toml::to_string(dt1Metadata).unwrap_or_else(|err| panic!("{err}"));
	write!(stdout, "{}\n{toml}", toml.len()).unwrap();
	let mut png = png::Encoder::new(stdout, image.width() as _, image.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&image.data).unwrap();

	struct Image {
		widthLog2: usize,
		height: usize,
		data: Vec<u8>,
	}
	impl Image {
		fn fromDT1(tiles: &[dt1::Tile], dt1: &[u8]) -> Self {
			const BLOCKWIDTH: usize = 32;
			const FLOOR_ROOF_BLOCKHEIGHT: usize = 15 + 1;
			const MAX_BLOCKHEIGHT: usize = 32;
			const MAX_BLOCKHEIGHT_LOG2: usize = log2!(MAX_BLOCKHEIGHT);
			const FLOOR: i32 = 0;
			const ROOF: i32 = 15;

			let _choices = {
				type T = usize;
				const N: usize = log2!(8192) - MAX_BLOCKHEIGHT_LOG2 + 1;
				const CHOICES: [T; N] = array_fromFn!(|i| 1 << (MAX_BLOCKHEIGHT_LOG2 + i));
				eprintln!("{:?}", CHOICES);
				Vec::<TileColumns>::with_capacity(CHOICES.len())
			};
			/*
			&mut TileColumns { fullColumnHeight: MAX_BLOCKHEIGHT, numFullColumns: 0, lastColumnHeight: 0 };
			for tile in tiles {
				let blockHeight =
					if matches!(tile.orientation, FLOOR | ROOF) { FLOOR_ROOF_BLOCKHEIGHT } else { MAX_BLOCKHEIGHT };
				for _ in &tile.blocks {
					while match square.putTile(blockHeight) {
						0 => false,
						excess if excess == blockHeight => true,
						excess => panic!("{excess} != {blockHeight}"),
					} {
						square.sizeLog2 += 1;
					}
				}
			}
			eprintln!("{0}x{0}", 1 << square.sizeLog2);
			*/

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
				requiredPixelArea.shrCeil(widthLog2).nextShlOf(log2!(MAX_BLOCKHEIGHT))
			};

			let mut image = Self { widthLog2, height, data: vec![0; height << widthLog2] };
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
					(if block.format == [1, 0] { Self::drawBlockIsometric } else { Self::drawBlockNormal })(
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
			assert_ne!(withValue, 0);
			self.data[atIndex] = withValue;
		}
	}

	Ok(())
}
