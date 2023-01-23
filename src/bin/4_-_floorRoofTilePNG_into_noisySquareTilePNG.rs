#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{
		dt1::{FLOOR_ROOF_TILEHEIGHT, TILEWIDTH},
		stdoutRaw, CopyExt, Image, TilesIterator, UsizeExt, Vec2, Vec2Ext, FULLY_TRANSPARENT, X, Y,
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
		const SQUARE_TILE_SIZE: usize = TILEWIDTH / 2;
		loop {
			let srcPoint = srcPoints.next(FLOOR_ROOF_TILEHEIGHT);
			if srcPoint[X] + TILEWIDTH > srcImage.width {
				break;
			}
			destImage.drawNoisySquareTile(destPoints.next(SQUARE_TILE_SIZE + 1), srcImage, srcPoint);
		}

		trait ImageExt {
			fn drawNoisySquareTile(&mut self, destPoint: Vec2, srcImage: &Self, srcPoint: Vec2);
		}
		impl ImageExt for Image {
			fn drawNoisySquareTile(&mut self, mut destPoint: Vec2, srcImage: &Self, mut srcPoint: Vec2) {
				destPoint[Y] += 1;
				srcPoint[X] += SQUARE_TILE_SIZE - 1;
				let [mut iY, mut jY] = [srcPoint[Y] * srcImage.width, destPoint[Y] * self.width];
				for Δx in 0..SQUARE_TILE_SIZE {
					let [mut i, mut j] = [srcPoint[X] + iY, destPoint[X] + jY];
					for Δy in 0..SQUARE_TILE_SIZE {
						match srcImage.data[i] {
							FULLY_TRANSPARENT => {}
							pixelValue => {
								assert_eq!(self.data[j], FULLY_TRANSPARENT);
								self.data[j] = pixelValue;
							}
						}
						i += usize::MAX + if Δy % 2 == 0 { 0 } else { srcImage.width };
						j += self.width;
					}
					destPoint.addAssign([
						1,
						(if Δx % 2 == 0 {
							srcPoint[X] += 2;
							usize::MAX
						} else {
							srcPoint[Y] += 1;
							iY += srcImage.width;
							1
						})
						.also(|&Δy| jY += self.width.mulSignumOf(Δy)),
					]);
				}
			}
		}
	}
	let mut png = png::Encoder::new(stdout, destImage.width as _, destImage.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
}
