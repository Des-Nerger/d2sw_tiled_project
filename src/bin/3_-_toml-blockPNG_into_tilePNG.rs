#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables)]

use {
	core::str::{self, FromStr},
	d2sw_tiled_project::{
		dt1::{self, DrawDestination},
		stdoutRaw, Image, FULLY_TRANSPARENT,
	},
	memchr::memchr,
	png::ColorType,
	std::io::{self, BufRead, BufWriter, Read},
};

fn main() {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let dt1Metadata: dt1::Metadata = {
		let (filesizeLine_len, filesize) = {
			let buffer = stdin.fill_buf().unwrap();
			let filesizeLine = str::from_utf8(&buffer[..=memchr(b'\n', buffer).unwrap()]).unwrap();
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
	let (srcImg, swappedPAL) = (&Image::fromPNG(png), png.info().palette.as_ref().unwrap().as_ref());
	let destImg =
		&mut Image { widthLog2: srcImg.widthLog2, data: vec![FULLY_TRANSPARENT; srcImg.data.len()] };
	let (mut x, mut y, height) = (0, 0, srcImg.height());
	for tile in &dt1Metadata.tiles {
		use dt1::{BLOCKWIDTH, FLOOR, FLOOR_ROOF_BLOCKHEIGHT, MAX_BLOCKHEIGHT, ROOF};
		let (blockHeight, blittedBlockHeight) = if matches!(tile.orientation, FLOOR | ROOF) {
			(FLOOR_ROOF_BLOCKHEIGHT, FLOOR_ROOF_BLOCKHEIGHT - 1)
		} else {
			(MAX_BLOCKHEIGHT, MAX_BLOCKHEIGHT)
		};
		for _ in &tile.blocks {
			let nextY = {
				let nextY = y + blockHeight;
				if nextY > height {
					x += BLOCKWIDTH;
					y = 0;
					blockHeight
				} else {
					nextY
				}
			};
			destImg.blitPixelsRectangle([x, y], [BLOCKWIDTH, blittedBlockHeight], srcImg, [x, y]);
			y = nextY;
		}
	}
	let mut png = png::Encoder::new(stdout, destImg.width() as _, destImg.height() as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(swappedPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&destImg.data).unwrap();

	/*
	let squares = &mut Vec::with_capacity([256, 512, 1024, 2048, 4096, 8192].len());
	const TILEWIDTH: usize = 160;
	squares.push(TilesSquare::<TILEWIDTH> { sizeLog2: log2!(256), usedHeight: 0 });
	let (mut i, mut mode, tiles, mut tileHeight) = (UNINIT, 0, &mut dt1Metadata.tiles.iter(), UNINIT);
	const UNINIT: usize = usize::MAX >> (usize::BITS as usize / 4);
	loop {
		match mode {
			0 => {
				tileHeight = match tiles.next() {
					None => break,
					Some(tile) => tile.height(),
				};
				println!("{tileHeight}");
			}
			_ => panic!("{}", mode),
		}
	}
	*/
}
