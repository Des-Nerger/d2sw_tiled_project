#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::{
		cmp::{max, min},
		mem,
		str::{self, FromStr},
	},
	d2sw_tiled_project::{
		dt1::{self, DrawDestination, BLOCKWIDTH, FLOOR_ROOF_BLOCKHEIGHT, TILEWIDTH},
		log2, stdoutRaw, Image, TileColumns, UsizeExt, Vec2Ext, FULLY_TRANSPARENT,
	},
	memchr::memchr,
	png::ColorType,
	std::{
		io::{self, BufRead, BufWriter, Read},
		process::ExitCode,
	},
};

fn main() -> ExitCode {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let mut dt1Metadata: dt1::Metadata = {
		let (filesizeLine_len, filesize) = {
			let buffer = stdin.fill_buf().unwrap();
			let filesizeLine = str::from_utf8(
				&buffer[..={
					match memchr(b'\n', buffer) {
						Some(index) => index,
						None => return ExitCode::FAILURE,
					}
				}],
			)
			.unwrap();
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
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let mut maxTileHeight = 0;
	{
		let srcPoints = &mut TilesIterator::<{ BLOCKWIDTH }>::new(srcImage);
		dt1Metadata.tiles.retain_mut(|tile| {
			if tile.blocks.len() == 0 {
				return false;
			}
			let (mut startY, mut endY, blockHeight) = (i16::MAX, i16::MIN, tile.blockHeight());
			for block in &tile.blocks {
				let (y, [startΔy, endΔy]) =
					(block.y, srcImage.ΔyBoundsᐸBLOCKWIDTHᐳ(srcPoints.next(blockHeight), blockHeight));
				startY = min(startY, y + startΔy);
				endY = max(endY, y + endΔy);
			}
			tile.height = ((endY - startY) as usize).nextMultipleOf(FLOOR_ROOF_BLOCKHEIGHT) as _;
			for block in &mut tile.blocks {
				block.y -= startY;
			}
			maxTileHeight = max(maxTileHeight, tile.height as usize);
			true
		});
	}
	let destImage = &mut {
		let height;
		let widthLog2 = {
			let chosenTileColumns = &{
				let choices = &mut Vec::<TileColumns>::new();
				choices.push(TileColumns {
					fullColumnHeight: maxTileHeight,
					numOverflownColumns: 0,
					lastColumnHeight: 0,
				});
				for tile in &dt1Metadata.tiles {
					choices.push(choices.last().unwrap().clone());
					let mut i = 0;
					while i < choices.len() {
						let result = choices[i].pushTile(tile.height as _);
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
					const WIDTH: usize = 0;
					pow2SquareSizes[A]
						.cmp(&pow2SquareSizes[B])
						.then_with(|| dimensions[B][WIDTH].cmp(&dimensions[A][WIDTH]))
				});
				mem::take(&mut choices[0])
			};
			let width;
			[width, height] = chosenTileColumns.dimensions(TILEWIDTH);
			let pow2Width = width.next_power_of_two();
			eprintln!(
				"{}; {}",
				format_args!("[{width}, {height}] --> [{pow2Width}, {height}]"),
				format_args!(
					"lastColumnHeight = {}, maxTileHeight = {maxTileHeight}",
					chosenTileColumns.lastColumnHeight,
				),
			);
			log2(pow2Width)
		};
		Image { widthLog2, data: vec![FULLY_TRANSPARENT; height << widthLog2] }
	};
	{
		let destPoints = &mut TilesIterator::<{ TILEWIDTH }>::new(destImage);
		let srcPoints = &mut TilesIterator::<{ BLOCKWIDTH }>::new(srcImage);
		for tile in &dt1Metadata.tiles {
			let (destPoint, blockHeight) = (destPoints.next(tile.height as _), tile.blockHeight());
			for block in &tile.blocks {
				destImage.blitPixelsRectangle(
					destPoint.add([block.x as _, block.y as _]),
					[BLOCKWIDTH, blockHeight],
					srcImage,
					srcPoints.next(blockHeight),
				);
			}
		}
	}
	let mut png = png::Encoder::new(stdout, destImage.width() as _, destImage.height() as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();

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

	ExitCode::SUCCESS
}
