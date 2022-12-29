#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::Parser,
	const_format::formatcp,
	core::str::FromStr,
	d2sw_tiled_project::{
		ds1::{self, existsTagLayer, ONE_SHADOW_LAYER},
		dt1::FLOOR_ORIENTATION,
		stdoutRaw, VecExt,
	},
	std::{
		collections::HashMap,
		io::{self, BufRead, Read, Write},
	},
};

fn main() {
	#[derive(Parser)]
	struct Args {
		#[clap(long)]
		skipWallLayers: bool,

		#[clap(long)]
		skipFloorLayers: bool,
	}

	let (Args { skipWallLayers, skipFloorLayers }, stdin, stdout) =
		(Args::parse(), io::stdin(), &mut io::BufWriter::new(stdoutRaw()));
	type Filesize = usize;
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
	let (stdin, filesizeLine, ds1, map) = (
		&mut stdin.lock(),
		&mut String::with_capacity(FILESIZE_LINE.len()),
		&mut Vec::new(),
		&mut HashMap::new(),
	);
	while {
		filesizeLine.clear();
		stdin.read_line(filesizeLine).unwrap() != 0
	} {
		let ds1::RootStruct { tagType, numWallLayers, layers, .. } = {
			let filesize = Filesize::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap();
			ds1.clear();
			ds1.reserve(filesize);
			ds1.setLen(filesize);
			stdin.read_exact(ds1).unwrap();
			match ds1::RootStruct::new(&mut io::Cursor::new(ds1 as &_)) {
				Err(_) => continue,
				Ok(ok) => ok,
			}
		};
		for i in 0..(layers.len() - ONE_SHADOW_LAYER - existsTagLayer(tagType) as usize) {
			let isWallLayer = i < numWallLayers as usize * 2;
			if isWallLayer & (skipWallLayers | (i % 2 == 1)) | !isWallLayer & skipFloorLayers {
				continue;
			}
			for (j, cell) in layers[i].iter().enumerate() {
				if cell & 0xFF != 0 {
					let key = [
						if isWallLayer { (layers[i + 1][j] & 0xFF) as u8 } else { FLOOR_ORIENTATION as u8 },
						(cell >> 20 & 0b11_1111) as u8,
						(cell >> 8 & 0b11_1111) as u8,
					];
					map.insert(key, map.get(&key).unwrap_or(&0) + 1);
				}
			}
		}
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
	let mut keys = Vec::from_iter(map.keys());
	keys.sort_by_key(|&key| map[key]); // not sure if sort_by_cached_key is worth it here
	for key in keys {
		writeln!(stdout, "{key:?}\t{}", map[key]).unwrap();
	}
}
