#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{
		dt1::{FLOOR_ROOF_TILEHEIGHT, SQUARE_TILE_SIZE, TILEWIDTH},
		stdoutRaw, Image, TilesIterator, X,
	},
	png::ColorType,
	std::io::{self, BufWriter},
};

fn main() {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let destImage = &mut Image::fromWidthHeight(
		srcImage.width / 2,
		srcImage.height /* + 1 */ / FLOOR_ROOF_TILEHEIGHT * (FLOOR_ROOF_TILEHEIGHT + 1),
	);
	{
		let srcPoints = &mut TilesIterator::<{ TILEWIDTH }>::new(srcImage);
		let destPoints = &mut TilesIterator::<{ SQUARE_TILE_SIZE }>::new(destImage);
		loop {
			let srcPoint = srcPoints.next(FLOOR_ROOF_TILEHEIGHT);
			if srcPoint[X] + TILEWIDTH > srcImage.width {
				break;
			}
			destImage.drawNoisySquareTile(destPoints.next(SQUARE_TILE_SIZE + 1), srcImage, srcPoint);
		}
	}
	let mut png = png::Encoder::new(stdout, destImage.width as _, destImage.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
}
