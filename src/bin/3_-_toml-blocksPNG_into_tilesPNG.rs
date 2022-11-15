#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	core::str::{self, FromStr},
	d2sw_tiled_project::{dt1, log2, TilesSquare},
	memchr::memchr,
	std::io::{self, BufRead, Read},
};

fn main() {
	let stdin = io::stdin();
	let stdin = &mut stdin.lock();
	let dt1Metadata: dt1::Metadata = {
		let (filesizeLine_len, filesize) = {
			let buffer = stdin.fill_buf().unwrap();
			let filesizeLine = str::from_utf8(&buffer[..=memchr(b'\n', buffer).unwrap()]).unwrap();
			(filesizeLine.len(), u64::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap())
		};
		stdin.consume(filesizeLine_len);
		toml::from_str(&io_readToString(stdin.take(filesize)).unwrap()).unwrap()
	};
	fn io_readToString<R: Read>(mut reader: R) -> io::Result<String> {
		let mut string = String::new();
		reader.read_to_string(&mut string)?;
		Ok(string)
	}

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
// let image = Image::fromPNG(&mut png::Decoder::new(stdin).unwrap()).unwrap();
