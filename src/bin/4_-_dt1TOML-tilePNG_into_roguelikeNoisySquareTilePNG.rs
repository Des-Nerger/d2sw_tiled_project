#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::{
		cmp::{max, min},
		mem,
		str::{self, FromStr},
	},
	d2sw_tiled_project::{
		dt1::{
			self, FLOOR_ORIENTATION, FLOOR_ROOF_BLOCKHEIGHT, FLOOR_ROOF_TILEHEIGHT, NUM_SUBTILES_PER_LINE,
			ROOF_ORIENTATION, SQUARE_SUBTILE_SIZE, SQUARE_TILE_SIZE, TILEWIDTH,
		},
		io_readToString, stdoutRaw, Image, TileColumns, TilesIterator, UsizeExt, Vec2Ext,
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
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	dt1Metadata.tiles.retain_mut(|tile| {
		if tile.blocks.len() == 0 {
			return false;
		}
		let (mut startY, mut endY, blockHeight) = (i16::MAX, i16::MIN, tile.blockHeight());
		for block in &tile.blocks {
			let (y, [startΔy, endΔy]) = (block.y, [0, blockHeight as _]);
			startY = min(startY, y + startΔy);
			endY = max(endY, y + endΔy);
		}
		tile.height = ((endY - startY) as usize).nextMultipleOf(FLOOR_ROOF_BLOCKHEIGHT) as _;
		true
	});
	let destImage = &mut {
		let (width, height);
		{
			let chosenTileColumns = &{
				let choices = &mut Vec::<TileColumns>::new();
				choices.push(TileColumns {
					fullColumnHeight: SQUARE_TILE_SIZE + 1,
					numOverflownColumns: 0,
					lastColumnHeight: 0,
				});
				for _ in &dt1Metadata.tiles {
					choices.push(choices.last().unwrap().clone());
					let mut i = 0;
					while i < choices.len() {
						let result = choices[i].pushTile(SQUARE_TILE_SIZE + 1);
						if i == choices.len() - 2 {
							let lastIndex = choices.len() - 1;
							if result == 0 {
								choices.truncate(lastIndex);
							} else {
								assert_eq!(choices[lastIndex].numOverflownColumns, 0);
								choices[lastIndex].fullColumnHeight += SQUARE_TILE_SIZE + 1;
								choices.push(choices[lastIndex].clone());
							}
						}
						i += 1;
					}
				}
				choices.sort_by(|a, b| {
					let dimensions = [a, b].map(|tileColumns| tileColumns.dimensions(SQUARE_TILE_SIZE));
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
			[width, height] = chosenTileColumns.dimensions(SQUARE_TILE_SIZE);
			eprintln!("[{width}, {height}]; lastColumnHeight = {}", chosenTileColumns.lastColumnHeight);
		}
		Image::fromWidthHeight(width, height)
	};
	{
		let destPoints = &mut TilesIterator::<{ SQUARE_TILE_SIZE }>::new(destImage);
		let srcPoints = &mut TilesIterator::<{ TILEWIDTH }>::new(srcImage);
		for tile in &dt1Metadata.tiles {
			let tileHeight = tile.height as usize;
			let [destPoint, srcPoint] = [destPoints.next(SQUARE_TILE_SIZE + 1), srcPoints.next(tileHeight)];
			if matches!(tile.orientation, FLOOR_ORIENTATION | ROOF_ORIENTATION)
				&& tileHeight == FLOOR_ROOF_TILEHEIGHT
			{
				destImage.drawNoisySquareTile(destPoint, srcImage, srcPoint);
			}
		}
	}
	{
		let destPoints = &mut TilesIterator::<{ SQUARE_TILE_SIZE }>::new(destImage);
		let hashSymbolImage = &Image::fromWidthData(
			SQUARE_SUBTILE_SIZE,
			Vec::from_iter(HASH_SYMBOL.iter().map(|&byte| 0_u8.wrapping_sub(byte))),
		);
		for tile in &dt1Metadata.tiles {
			let (mut i, mut destPoint) = (
				0,
				destPoints.next(SQUARE_TILE_SIZE + 1).add([0, (NUM_SUBTILES_PER_LINE - 1) * SQUARE_SUBTILE_SIZE]),
			);
			/*
			if matches!(tile.orientation, FLOOR_ORIENTATION | ROOF_ORIENTATION) {
				continue;
			}
			*/
			for _ in 0..NUM_SUBTILES_PER_LINE {
				for _ in 0..NUM_SUBTILES_PER_LINE {
					if tile.subtileFlags[i] & (BLOCK_WALK | BLOCK_PLAYER_WALK) != 0 {
						destImage.blitPixelsRectangle(
							destPoint,
							[SQUARE_SUBTILE_SIZE, SQUARE_SUBTILE_SIZE + 1],
							hashSymbolImage,
							[0, 0],
						);
					}
					i += 1;
					destPoint = destPoint.add([SQUARE_SUBTILE_SIZE, 0]);
				}
				destPoint = destPoint.add([
					0_usize.wrapping_sub(NUM_SUBTILES_PER_LINE * SQUARE_SUBTILE_SIZE),
					0_usize.wrapping_sub(SQUARE_SUBTILE_SIZE),
				]);
			}
			const BLOCK_WALK: u8 = 0b0001;
			const BLOCK_PLAYER_WALK: u8 = 0b1000;
		}
		#[rustfmt::skip]
		const HASH_SYMBOL: [u8; SQUARE_SUBTILE_SIZE * (SQUARE_SUBTILE_SIZE + 1)] = [
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		/*
			0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
		*/
		];
	}
	let mut png = png::Encoder::new(stdout, destImage.width as _, destImage.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
	ExitCode::SUCCESS
}
