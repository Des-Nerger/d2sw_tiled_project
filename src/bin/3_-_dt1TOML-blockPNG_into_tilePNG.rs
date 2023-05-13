#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::Parser,
	core::{
		cmp::max,
		mem,
		str::{self, FromStr},
	},
	d2sw_tiled_project::{
		dt1::{self, BLOCKWIDTH, FLOOR_ROOF_BLOCKHEIGHT, TILEWIDTH},
		io_readToString, stdoutRaw, Image, MinAssign_MaxAssign_Ext, TileColumns, TilesIterator, UsizeExt,
		Vec2Ext, WIDTH,
	},
	memchr::memchr,
	png::ColorType,
	std::{
		io::{self, BufRead, BufWriter, Read},
		process::ExitCode,
	},
};

fn main() -> ExitCode {
	#[derive(Parser)]
	struct Args {
		#[clap(long)]
		zealousVerticalPacking: bool,
	}
	let Args { zealousVerticalPacking } = Args::parse();

	let stdin = &mut io::stdin().lock();
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
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let mut maxTileHeight = 0_usize;
	{
		let srcPoints = &mut TilesIterator::new(BLOCKWIDTH, srcImage);
		// eprintln!();
		dt1Metadata.tiles.retain_mut(|tile| {
			if tile.blocks.len() == 0 {
				return false;
			}
			let (mut startY, mut endY, blockHeight) = (i16::MAX, i16::MIN, tile.blockHeight());
			for block in &tile.blocks {
				let (y, [startΔy, endΔy]) = (
					block.y,
					if zealousVerticalPacking {
						srcImage.boundingΔyRangeᐸBLOCKWIDTHᐳ(srcPoints.next(blockHeight), blockHeight)
					} else {
						[0, blockHeight as _]
					},
				);
				startY.minAssign(y + startΔy);
				endY.maxAssign(y + endΔy);
			}
			// eprintln!("startY={startY}, endY={endY}");
			tile.height = ((endY - startY) as usize).nextMultipleOf(FLOOR_ROOF_BLOCKHEIGHT) as _;
			for block in &mut tile.blocks {
				block.y -= startY;
			}
			maxTileHeight.maxAssign(tile.height as _);
			true
		});
	}
	let destImage = &mut {
		let (width, height);
		{
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
					pow2SquareSizes[A]
						.cmp(&pow2SquareSizes[B])
						.then_with(|| dimensions[B][WIDTH].cmp(&dimensions[A][WIDTH]))
				});
				mem::take(&mut choices[0])
			};
			[width, height] = chosenTileColumns.dimensions(TILEWIDTH);
			eprintln!(
				"[{width}, {height}]; lastColumnHeight = {}, maxTileHeight = {maxTileHeight}",
				chosenTileColumns.lastColumnHeight,
			);
		}
		Image::fromWidthHeight(width, height)
	};
	{
		let destPoints = &mut TilesIterator::new(TILEWIDTH, destImage);
		let srcPoints = &mut TilesIterator::new(BLOCKWIDTH, srcImage);
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
	let mut png =
		png::Encoder::new(BufWriter::new(stdoutRaw()), destImage.width as _, destImage.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
	ExitCode::SUCCESS
}
