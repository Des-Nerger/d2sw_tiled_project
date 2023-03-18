#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::Parser,
	core::{array, ops::RangeInclusive},
	d2sw_tiled_project::{
		dt1::{FLOOR_ROOF_TILEHEIGHT, NBPIX, TILEWIDTH, XJUMP},
		stdoutRaw, unlet, Image, TilesIterator, UsizeExt, X, Y,
	},
	png::ColorType,
	std::io::{self, BufWriter},
};

fn main() {
	#[derive(Parser)]
	struct Args {
		#[clap(long, default_value_t = 0)]
		tileDimensionsBitshiftLeftBy: usize,
	}
	let Args { tileDimensionsBitshiftLeftBy } = Args::parse();
	let [tilewidth, floorRoofTileheight] =
		[TILEWIDTH, FLOOR_ROOF_TILEHEIGHT].map(|dimension| dimension << tileDimensionsBitshiftLeftBy);
	eprintln!("{:?}", [tilewidth, floorRoofTileheight],);

	let stdin = &mut io::stdin().lock();
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let (srcImage, pngPAL) = (&mut Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let destImage = &mut Image::fromWidthHeight(
		(srcImage.width - tilewidth / 2) * 2,
		srcImage.height - floorRoofTileheight / 2,
	);
	{
		let destPoints = &mut TilesIterator::new(tilewidth, destImage);
		loop {
			let destPoint = destPoints.next(floorRoofTileheight);
			if destPoint[X] + tilewidth > destImage.width {
				break;
			}
			let [mut i, mut j, mut xjump, mut nbpix] = [
				(destPoint[Y]
					+ if destPoints.tileColumns.numOverflownColumns % 2 == 0 { 0 } else { floorRoofTileheight / 2 })
					* srcImage.width
					+ destPoint[X] / 2,
				destPoint[Y] * destImage.width + destPoint[X],
				(tilewidth - NBPIX[0]) / 2,
				NBPIX[0],
			];
			let floorRoofTile_effectiveHeight = floorRoofTileheight - 1;
			unlet!(floorRoofTileheight);
			for signedRow in RangeInclusive::fromArray(array::from_fn(|i| {
				(0_usize.wrapping_sub(floorRoofTile_effectiveHeight / 2)).mulSignumOf(0_usize.wrapping_sub(i))
					as isize
			})) {
				destImage.data[j + xjump..][..nbpix].copy_from_slice(&srcImage.data[i + xjump..][..nbpix]);
				i += srcImage.width;
				j += destImage.width;
				xjump += (XJUMP[0] - XJUMP[1]).mulSignumOf(signedRow as _);
				nbpix += NBPIX[0].wrapping_sub(NBPIX[1]).mulSignumOf(signedRow as _);
			}
			trait RangeExt<Idx> {
				fn fromArray(bounds: [Idx; 2]) -> Self;
			}
			impl<Idx> RangeExt<Idx> for RangeInclusive<Idx> {
				fn fromArray([start, end]: [Idx; 2]) -> Self {
					start..=end
				}
			}
		}
	}
	let mut png =
		png::Encoder::new(BufWriter::new(stdoutRaw()), destImage.width as _, destImage.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
}
