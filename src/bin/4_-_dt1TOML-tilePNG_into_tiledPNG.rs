#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::{
		cmp::{max, min},
		str::{self, FromStr},
	},
	d2sw_tiled_project::{
		applyMacro,
		dt1::{self, Block, TILEWIDTH},
		io_readToString, stdoutRaw, unlet, Image, TilesIterator, X, Y,
	},
	memchr::memchr,
	png::ColorType,
	std::{
		io::{self, BufRead, BufWriter, Read},
		process::ExitCode,
	},
};

fn main() -> ExitCode {
	let stdin = &mut io::stdin().lock();
	let tiles = &{
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
		toml::from_str::<dt1::Metadata>(&io_readToString(stdin.take(filesize)).unwrap()).unwrap()
	}
	.tiles;
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	applyMacro!(unlet; (png), (stdin));
	let (mut destTileHeight, srcTileHeights) = (0, &mut Vec::with_capacity(tiles.len()));
	for tile in tiles {
		let [mut startY, mut endY, blockHeight] = [i16::MAX, i16::MIN, tile.blockHeight() as _];
		for &Block { y, .. } in &tile.blocks {
			startY = min(startY, y);
			endY = max(endY, y + blockHeight);
		}
		let srcTileHeight = (endY - startY) as usize;
		srcTileHeights.push(srcTileHeight);
		destTileHeight = max(destTileHeight, srcTileHeight);
	}
	{
		let destColumnCount =
			(tiles.len() as f32 * (destTileHeight as f32 / TILEWIDTH as f32)).sqrt().ceil() as usize;
		let destRowCount = ((tiles.len() - 1) / destColumnCount) + 1;
		let destImage =
			&mut Image::fromWidthHeight(destColumnCount * TILEWIDTH, destRowCount * destTileHeight);
		{
			let (mut destPoint, srcTileHeights, srcPoints) =
				([0, 0], &mut srcTileHeights.iter(), &mut TilesIterator::new(TILEWIDTH, srcImage));
			'outer: for _ in 0..destRowCount {
				for _ in 0..destColumnCount {
					if let Some(&srcTileHeight) = srcTileHeights.next() {
						destImage.blitPixelsRectangle(
							destPoint,
							[TILEWIDTH, srcTileHeight],
							srcImage,
							srcPoints.next(srcTileHeight),
						);
						destPoint[X] += TILEWIDTH;
					} else {
						break 'outer;
					};
				}
				destPoint = [0, destPoint[Y] + destTileHeight];
			}
		}
		let mut png =
			png::Encoder::new(BufWriter::new(stdoutRaw()), destImage.width as _, destImage.height as _);
		png.set_color(ColorType::Indexed);
		png.set_palette(pngPAL);
		png.set_trns(&[0][..]);
		png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
	}
	eprintln!("\"tileheight\":{destTileHeight}");
	ExitCode::SUCCESS
}
