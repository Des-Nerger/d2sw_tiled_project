#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables)]

use {
	core::{
		cmp::{
			max,
			Ordering::{Greater, Less},
		},
		mem,
		str::{self, FromStr},
	},
	d2sw_tiled_project::{
		dt1::{self, DrawDestination, BLOCKWIDTH, FLOOR_ROOF_BLOCKHEIGHT, MAX_TILEHEIGHT, TILEWIDTH},
		log2, stdoutRaw, Image, TileColumns, Vec2Ext, FULLY_TRANSPARENT,
	},
	memchr::memchr,
	png::ColorType,
	std::io::{self, BufRead, BufWriter, Read},
};

fn main() {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let dt1Metadata: dt1::Metadata = {
		let (filesizeLine_len, filesize) = {
			let buffer = stdin.fill_buf().unwrap();
			let filesizeLine = str::from_utf8(&buffer[..=memchr(b'\n', buffer).unwrap()]).unwrap();
			(filesizeLine.len(), u64::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap())
		};
		stdin.consume(filesizeLine_len);
		toml::from_str(&io_readToString(stdin.take(filesize)).unwrap()).unwrap()
	};
	fn io_readToString(mut reader: impl Read) -> io::Result<String> {
		let mut string = String::new();
		reader.read_to_string(&mut string)?;
		Ok(string)
	}
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImg, swappedPAL) = (&Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let destImg = &mut {
		let height;
		let widthLog2 = {
			let chosenTileColumns = &{
				let choices = &mut Vec::<TileColumns>::new();
				choices.push(TileColumns {
					fullColumnHeight: MAX_TILEHEIGHT,
					numOverflownColumns: 0,
					lastColumnHeight: 0,
				});
				for tile in &dt1Metadata.tiles {
					choices.push(choices.last().unwrap().clone());
					let mut i = 0;
					while i < choices.len() {
						let result = choices[i].pushTile(tile.height_y0_blockHeight()[0]);
						if i == choices.len() - 2 {
							let lastIndex = choices.len() - 1;
							if result == 0 {
								choices.truncate(lastIndex);
							} else {
								assert_eq!(choices[lastIndex].numOverflownColumns, 0);
								choices[lastIndex].fullColumnHeight += FLOOR_ROOF_BLOCKHEIGHT;
								choices.push(choices[lastIndex].clone());
							}
						}
						i += 1;
					}
				}
				choices.sort_by(|a, b| {
					let dimensions = [a, b].map(|tileColumns| tileColumns.dimensions(TILEWIDTH));
					let pow2SquareSizes = dimensions.map(|[width, height]| max(width, height).next_power_of_two());
					const A: usize = 0;
					const B: usize = 1;
					match (pow2SquareSizes[A].wrapping_sub(pow2SquareSizes[B]) as isize).signum() {
						-1 => return Less,
						1 => return Greater,
						_ => {}
					}
					const WIDTH: usize = 0;
					dimensions[B][WIDTH].cmp(&dimensions[A][WIDTH])
				});
				mem::take(&mut choices[0])
			};
			let width;
			[width, height] = chosenTileColumns.dimensions(TILEWIDTH);
			let pow2Width = width.next_power_of_two();
			eprintln!(
				"[{width}, {height}] --> [{pow2Width}, {height}]; lastColumnHeight = {}",
				chosenTileColumns.lastColumnHeight,
			);
			log2(pow2Width)
		};
		Image { widthLog2, data: vec![FULLY_TRANSPARENT; height << widthLog2] }
	};
	{
		let destPoints = &mut TilesIterator::<{ TILEWIDTH }>::new(destImg);
		let srcPoints = &mut TilesIterator::<{ BLOCKWIDTH }>::new(srcImg);
		for tile in &dt1Metadata.tiles {
			let [tileHeight, y0, blockHeight] = tile.height_y0_blockHeight();
			let destPoint = destPoints.next(tileHeight).add([0, y0]);
			for block in &tile.blocks {
				destImg.blitPixelsRectangle(
					destPoint.add([block.x as _, block.y as _]),
					[BLOCKWIDTH, blockHeight],
					srcImg,
					srcPoints.next(blockHeight),
				);
			}
		}

		struct TilesIterator<const TILEWIDTH: usize>(TileColumns);
		impl<const TILEWIDTH: usize> TilesIterator<TILEWIDTH> {
			#[inline(always)]
			fn new(image: &Image) -> Self {
				Self(TileColumns {
					fullColumnHeight: image.data.len() >> image.widthLog2,
					numOverflownColumns: 0,
					lastColumnHeight: 0,
				})
			}
			#[inline(always)]
			fn next(&mut self, tileHeight: usize) -> [usize; 2] {
				let tileColumns = self.0.clone();
				if self.0.pushTile(tileHeight) != 0 {
					[self.0.numOverflownColumns * TILEWIDTH, 0]
				} else {
					[tileColumns.numOverflownColumns * TILEWIDTH, tileColumns.lastColumnHeight]
				}
			}
		}
	}
	let mut png = png::Encoder::new(stdout, destImg.width() as _, destImg.height() as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImg.data).unwrap();
}
