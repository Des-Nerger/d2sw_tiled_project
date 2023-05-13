#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::Parser,
	const_format::formatcp,
	core::str::FromStr,
	d2sw_tiled_project::{
		ds1::{
			self, existsTagLayer, LAYER_DRAWING_PRIORITY_MASK, MAIN_INDEX_OFFSET, ONE_SHADOW_LAYER,
			SUB_INDEX_OFFSET,
		},
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
	type Filesize = usize;
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
	let componentMaxValue;
	let (stdin, stdout, counts, filesizeLine, ds1) = &mut (
		io::stdin().lock(),
		io::BufWriter::new(stdoutRaw()),
		{
			componentMaxValue = match componentType {
				Orientation => ds1::ORIENTATION_MASK,
				MainIndex => ds1::MAIN_INDEX_MAX,
				SubIndex => ds1::SUB_INDEX_MAX,
			};
			vec![0; (1 + componentMaxValue) as _].into_boxed_slice()
		},
		String::with_capacity(FILESIZE_LINE.len()),
		Vec::new(),
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
					counts[match componentType {
						Orientation => {
							if isWallLayer {
								layers[i + 1][j] & componentMaxValue
							} else {
								FLOOR_ORIENTATION as u32
							}
						}
						MainIndex => cell >> MAIN_INDEX_OFFSET & componentMaxValue,
						SubIndex => cell >> SUB_INDEX_OFFSET & componentMaxValue,
					} as usize] += 1;
				}
			}
		}
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
	let mut indices = Vec::from_iter(0..=componentMaxValue as u8).into_boxed_slice();
	indices.sort_by_key(|&i| counts[i as usize]);
	for &i in indices.into_iter() {
		writeln!(stdout, "{i}\t{}", counts[i as usize]).unwrap();
	}
}
