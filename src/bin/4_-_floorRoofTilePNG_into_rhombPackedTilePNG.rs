#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{
		dt1::{DrawDestination, FLOOR_ROOF_TILEHEIGHT, TILEWIDTH},
		stdoutRaw, Image, TilesIterator, FULLY_TRANSPARENT, X, Y,
	},
	png::ColorType,
	std::io::{self, BufWriter},
};

fn main() {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let destImage = &mut Image {
		widthLog2: srcImage.widthLog2,
		data: vec![FULLY_TRANSPARENT; (srcImage.height() + FLOOR_ROOF_TILEHEIGHT / 2) << srcImage.widthLog2],
	};
	{
		let srcPoints = &mut TilesIterator::<{ TILEWIDTH }>::new(srcImage);
		loop {
			let srcPoint = srcPoints.next(FLOOR_ROOF_TILEHEIGHT);
			if srcPoint[X] + TILEWIDTH > srcImage.width() {
				break;
			}
			destImage.blitPixelsRectangle(
				[
					srcPoint[X] / 2,
					srcPoint[Y]
						+ if srcPoints.0.numOverflownColumns % 2 == 0 { 0 } else { FLOOR_ROOF_TILEHEIGHT / 2 },
				],
				[TILEWIDTH, FLOOR_ROOF_TILEHEIGHT],
				srcImage,
				srcPoint,
			);
		}
	}
	let mut png = png::Encoder::new(stdout, destImage.width() as _, destImage.height() as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
}
