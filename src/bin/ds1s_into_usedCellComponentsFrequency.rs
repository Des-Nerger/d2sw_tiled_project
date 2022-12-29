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
	std::io::{self, BufRead, Read, Write},
};

fn main() {
	#[derive(Parser)]
	struct Args {
		#[clap(long)]
		skipWallLayers: bool,

		#[clap(long)]
		skipFloorLayers: bool,

		#[clap(arg_enum)]
		cellComponentType: CellComponentType,
	}
	#[derive(Clone, Debug, clap::ValueEnum)]
	enum CellComponentType {
		Orientation,
		MainIndex,
		SubIndex,
	}
	use CellComponentType::*;

	let Args { skipWallLayers, skipFloorLayers, cellComponentType: componentType } = Args::parse();
	let (componentMaxValue, ref mut counts) = {
		let numValues = {
			1 << match componentType {
				Orientation => u8::BITS,
				MainIndex => 6,
				SubIndex => 6,
			}
		};
		((numValues - 1) as u32, vec![0; numValues])
	};

	let (stdin, stdout) = (io::stdin(), &mut io::BufWriter::new(stdoutRaw()));
	type Filesize = usize;
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
	let (stdin, filesizeLine, ds1) =
		(&mut stdin.lock(), &mut String::with_capacity(FILESIZE_LINE.len()), &mut Vec::new());
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
				const LAYER_DRAWING_MAX_PRIORITY: u32 = u8::MAX as _;
				if cell & LAYER_DRAWING_MAX_PRIORITY != 0 {
					counts[match componentType {
						Orientation => {
							if isWallLayer {
								layers[i + 1][j] & componentMaxValue
							} else {
								FLOOR_ORIENTATION as u32
							}
						}
						MainIndex => cell >> 20 & componentMaxValue,
						SubIndex => cell >> 8 & componentMaxValue,
					} as usize] += 1;
				}
			}
		}
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
	let mut indices = Vec::from_iter(0..=componentMaxValue as u8);
	indices.sort_by_key(|&i| counts[i as usize]);
	for &i in &indices {
		writeln!(stdout, "{i}\t{}", counts[i as usize]).unwrap();
	}
}
