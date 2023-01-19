#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{
		dt1::{DrawDestination, FLOOR_ROOF_TILEHEIGHT, TILEWIDTH},
		stdoutRaw, Image, TilesIterator, Vec2, Vec2Ext, FULLY_TRANSPARENT, X, Y,
	},
	png::ColorType,
	std::io::{self, BufWriter},
};

fn main() {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let destImage = &mut {
		let widthLog2 = srcImage.widthLog2 - 1;
		Image {
			widthLog2,
			data: vec![
				FULLY_TRANSPARENT;
				(srcImage.height() /* / FLOOR_ROOF_TILEHEIGHT * (FLOOR_ROOF_TILEHEIGHT + 1) */ + 1)
					<< widthLog2
			],
		}
	};
	{
		let srcPoints = &mut TilesIterator::<{ TILEWIDTH }>::new(srcImage);
		let destPoints = &mut TilesIterator::<{ SQUARE_TILE_SIZE }>::new(destImage);
		const SQUARE_TILE_SIZE: usize = TILEWIDTH / 2;
		loop {
			let srcPoint = srcPoints.next(FLOOR_ROOF_TILEHEIGHT);
			if srcPoint[X] + TILEWIDTH > srcImage.width() {
				break;
			}
			destImage.drawSquareTile(destPoints.next(SQUARE_TILE_SIZE /* + 1 */), srcImage, srcPoint);
		}

		trait ImageExt {
			fn drawSquareTile(&mut self, destPoint: Vec2, srcImage: &Self, srcPoint: Vec2);
		}
		impl ImageExt for Image {
			fn drawSquareTile(&mut self, mut destPoint: Vec2, srcImage: &Self, mut srcPoint: Vec2) {
				destPoint[Y] += 1;
				srcPoint[X] += SQUARE_TILE_SIZE - 1;
				let ΔiNextLine = 1 << srcImage.widthLog2;
				let ΔjNextLine = 1 << self.widthLog2;
				for Δx in 0..SQUARE_TILE_SIZE {
					let mut i = srcPoint[X] + (srcPoint[Y] << srcImage.widthLog2);
					let mut j = destPoint[X] + (destPoint[Y] << self.widthLog2);
					for Δy in 0..SQUARE_TILE_SIZE {
						match srcImage.data[i] {
							FULLY_TRANSPARENT => {}
							pixelValue => {
								assert_eq!(self.data[j], FULLY_TRANSPARENT);
								self.data[j] = pixelValue;
							}
						}
						i += usize::MAX + if Δy % 2 == 0 { 0 } else { ΔiNextLine };
						j += ΔjNextLine;
					}
					destPoint.addAssign([
						1,
						if Δx % 2 == 0 {
							srcPoint[X] += 2;
							usize::MAX
						} else {
							srcPoint[Y] += 1;
							1
						},
					]);
				}
			}
		}
	}
	let mut png = png::Encoder::new(stdout, destImage.width() as _, destImage.height() as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
}
