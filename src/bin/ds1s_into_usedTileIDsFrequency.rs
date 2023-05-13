#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::Parser,
	const_format::formatcp,
	core::str::FromStr,
	d2sw_tiled_project::{
		ds1::{
			self, existsTagLayer, LAYER_DRAWING_PRIORITY_MASK, MAIN_INDEX_MAX, MAIN_INDEX_OFFSET,
			ONE_SHADOW_LAYER, ORIENTATION_MASK, SUB_INDEX_MAX, SUB_INDEX_OFFSET,
		},
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
	let Args { skipWallLayers, skipFloorLayers } = Args::parse();

	type Filesize = usize;
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
	let (stdin, stdout, filesizeLine, ds1, hashMap) = &mut (
		io::stdin().lock(),
		io::BufWriter::new(stdoutRaw()),
		String::with_capacity(FILESIZE_LINE.len()),
		Vec::new(),
		HashMap::new(),
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
				if cell & LAYER_DRAWING_PRIORITY_MASK != 0 {
					let key: [u8; 3] = [
						if isWallLayer { (layers[i + 1][j] & ORIENTATION_MASK) as _ } else { FLOOR_ORIENTATION as _ },
						(cell >> MAIN_INDEX_OFFSET & MAIN_INDEX_MAX) as _,
						(cell >> SUB_INDEX_OFFSET & SUB_INDEX_MAX) as _,
					];
					hashMap.insert(key, hashMap.get(&key).unwrap_or(&0) + 1);
				}
			}
		}
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
	let mut keys = Vec::from_iter(hashMap.keys()).into_boxed_slice();
	keys.sort_by_key(|&key| hashMap[key]); // not sure if sort_by_cached_key is worth it here
	for &key in keys.into_iter() {
		writeln!(stdout, "{key:?}\t{}", hashMap[key]).unwrap();
	}
}
