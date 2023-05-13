#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::Parser,
	const_format::formatcp,
	core::{
		array, iter,
		num::ParseIntError,
		str::{self, FromStr},
	},
	d2sw_tiled_project::{
		applyMacro, default, stdoutRaw, toml_toStringPretty, DivRemExt, Image, LenConst_Ext,
		MinAssign_MaxAssign_Ext, Rectangle, Vec2, Vec2Ext, DIMENSIONS, HEIGHT, POINT, WIDTH, X, Y,
	},
	png::ColorType,
	serde::Serialize,
	std::{
		io::{self, BufRead, BufWriter, Read, Write},
		process::Command,
	},
};

fn main() {
	#[derive(Parser, Debug)]
	struct Args {
		#[clap(long)]
		firstgid: usize,
		#[clap(required = true)]
		tileDimensionPairs: Vec<DimensionPair>,
		rectpackerPath: String,
	}
	#[derive(Debug)]
	struct DimensionPair([usize; 2]);
	impl FromStr for DimensionPair {
		type Err = ParseIntError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			let mut pair = [default(); 2];
			for (i, s) in s.split('x').enumerate() {
				pair[i] = s.parse()?;
			}
			Ok(Self(pair))
		}
	}
	let Args { firstgid, tileDimensionPairs, rectpackerPath } = Args::parse();
	let (pngPAL, srcImages, srcRects) = {
		let (mut pngPAL, mut srcImages) = (None, Vec::with_capacity(tileDimensionPairs.len()));
		{
			type Filesize = usize;
			const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
			let (mut i, stdin, filesizeLine) =
				(0, &mut io::stdin().lock(), &mut String::with_capacity(FILESIZE_LINE.len()));
			while i < tileDimensionPairs.len() && {
				filesizeLine.clear();
				stdin.read_line(filesizeLine).unwrap() != 0
			} {
				let png = &mut png::Decoder::new(
					stdin.take(Filesize::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap() as _),
				)
				.read_info()
				.unwrap();
				let pngInfo = png.info();
				assert_eq!(pngInfo.color_type, ColorType::Indexed);
				{
					let cow = pngInfo.palette.as_ref().unwrap();
					if let Some(pngPAL) = &pngPAL {
						assert_eq!(pngPAL, cow.as_ref());
					} else {
						pngPAL = Some(cow.clone().into_owned());
					}
				}
				srcImages.push(Image::fromPNG(png));
				i += 1;
			}
			assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
		}
		let (mut gid, mut srcRects, backgroundTileCenter) =
			(firstgid, Vec::<(usize, usize, Rectangle, Vec2)>::new(), tileDimensionPairs[0].0.div(2));
		for (i, (srcImage, &DimensionPair(tileDimensions))) in
			iter::zip(srcImages.iter(), tileDimensionPairs[tileDimensionPairs.len() - srcImages.len()..].iter())
				.enumerate()
		{
			macro_rules! letLines {
				($lines: ident, $dimension: ident, $DIMENSION: ident) => {
					let $lines;
					{
						let rem;
						[$lines, rem] = srcImage.$dimension.div_rem(tileDimensions[$DIMENSION]);
						assert_eq!(rem, 0);
					}
				};
			}
			applyMacro!(letLines; (rows, height, HEIGHT), (columns, width, WIDTH));
			srcRects.reserve(rows * columns);
			let [mut point, offset] = [[0; 2], tileDimensions.add(backgroundTileCenter.neg())];
			for _ in 0..rows {
				for _ in 0..columns {
					if let Some(boundingRectangle) = srcImage.boundingRectangle([point, tileDimensions]) {
						srcRects.push((
							gid,
							i,
							boundingRectangle,
							offset.add(point.add(boundingRectangle[POINT].neg())),
						));
					}
					gid += 1;
					point[X] += tileDimensions[WIDTH];
				}
				point = [0, point[Y] + tileDimensions[HEIGHT]];
			}
		}
		eprintln!("srcRects.len() == {}", srcRects.len());
		(pngPAL.unwrap(), &mut srcImages.into_boxed_slice(), &mut srcRects.into_boxed_slice())
	};
	let mut destPoints = Vec::with_capacity(srcRects.len());
	let destImage = &mut {
		let (mut destImageDimensions, srcRectDimensions) =
			([0_usize; 2], &mut Vec::with_capacity(srcRects.len() * Vec2::LEN));
		for (_, _, srcRect, _) in srcRects.iter() {
			for dimension in srcRect[DIMENSIONS] {
				srcRectDimensions.push(dimension.to_string());
			}
		}
		for (i, line) in str::from_utf8(
			&Command::new(&rectpackerPath)
				.args(srcRectDimensions)
				.output()
				.unwrap_or_else(|err| panic!("{:?}: {err}", rectpackerPath))
				.stdout,
		)
		.unwrap()
		.lines()
		.enumerate()
		{
			let destPoint: Vec2 = array::from_fn({
				let mut coords = line.split(' ');
				move |_| usize::from_str(coords.next().unwrap()).unwrap()
			});
			for (j, dimension) in destPoint.add(srcRects[i].2[DIMENSIONS]).into_iter().enumerate() {
				destImageDimensions[j].maxAssign(dimension);
			}
			destPoints.push(destPoint);
		}
		Image::fromWidthHeight(destImageDimensions[WIDTH], destImageDimensions[HEIGHT])
	};
	type AtlasDefs = Vec<(usize, usize, usize, usize, usize, isize, isize)>;
	let mut atlasDefs = AtlasDefs::with_capacity(srcRects.len());
	for (&(gid, i, [srcPoint, rectDimensions], offset), destPoint) in
		iter::zip(srcRects.into_iter(), destPoints.into_iter())
	{
		atlasDefs.push((
			gid,
			destPoint[X],
			destPoint[Y],
			rectDimensions[WIDTH],
			rectDimensions[HEIGHT],
			offset[X] as _,
			offset[Y] as _,
		));
		destImage.blitPixelsRectangle(destPoint, rectDimensions, &srcImages[i], srcPoint);
	}
	#[derive(Serialize)]
	struct AtlasDefsTOML {
		#[serde(rename = "_.png")]
		pngFilePath: AtlasDefs,
	}
	let (stdout, tomlString) = (
		&mut BufWriter::new(stdoutRaw()),
		toml_toStringPretty(&AtlasDefsTOML { pngFilePath: atlasDefs }).unwrap_or_else(|err| panic!("{err}")),
	);
	write!(stdout, "{}\n{tomlString}", tomlString.len()).unwrap();
	let mut png = png::Encoder::new(stdout, destImage.width as _, destImage.height as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImage.data).unwrap();
}
