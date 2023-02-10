#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::{value_parser, Parser},
	d2sw_tiled_project::{
		ds1::{
			self, existsTagLayer, LAYER_DRAWING_PRIORITY_MASK, MAIN_INDEX_MASK, MAIN_INDEX_MAX,
			MAIN_INDEX_OFFSET, ONE_SHADOW_LAYER, SUB_INDEX_MASK, SUB_INDEX_MAX, SUB_INDEX_OFFSET,
		},
		io_readToString, stdoutRaw, toml_toStringPretty,
	},
	std::io::{self, Write},
};

fn main() {
	#[derive(Parser)]
	struct Args {
		#[clap(value_parser = value_parser!(u32).range(0..=(MAIN_INDEX_MAX as _)))]
		mainIndex: u32,

		#[clap(value_parser = value_parser!(u32).range(0..=(SUB_INDEX_MAX as _)))]
		subIndex: u32,
	}
	let (setID, ds1RootStruct) = (
		{
			let Args { mainIndex, subIndex } = Args::parse();
			mainIndex << MAIN_INDEX_OFFSET | subIndex << SUB_INDEX_OFFSET
		},
		&mut toml::from_str(&io_readToString(io::stdin()).unwrap()).unwrap(),
	);
	let &mut ds1::RootStruct { tagType, numWallLayers, ref mut layers, .. } = ds1RootStruct;
	for layer in {
		let len = layers.len();
		&mut layers[((numWallLayers * 2) as _)..(len - ONE_SHADOW_LAYER - existsTagLayer(tagType) as usize)]
	} {
		for cell in layer {
			*cell = {
				let &mut cell = cell;
				if cell & LAYER_DRAWING_PRIORITY_MASK == 0 {
					continue;
				}
				cell & !(MAIN_INDEX_MASK | SUB_INDEX_MASK) | setID
			};
		}
	}
	stdoutRaw()
		.write_all(&toml_toStringPretty(ds1RootStruct).unwrap_or_else(|err| panic!("{err}")).into_bytes())
		.unwrap();
}
