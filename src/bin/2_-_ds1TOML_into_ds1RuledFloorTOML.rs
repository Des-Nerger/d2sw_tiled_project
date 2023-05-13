#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	clap::{value_parser, Parser},
	core::array,
	d2sw_tiled_project::{
		ds1::{
			self, LAYER_DRAWING_PRIORITY_MASK, MAIN_INDEX_MASK, MAIN_INDEX_MAX, MAIN_INDEX_OFFSET,
			SUB_INDEX_MASK, SUB_INDEX_OFFSET,
		},
		io_readToString, stdoutRaw, toml_toStringPretty, MinAssign_MaxAssign_Ext,
	},
	std::io::{self, Write},
};

fn main() {
	let (offsettedMainIndex, ds1RootStruct) = (
		{
			#[derive(Parser)]
			struct Args {
				#[clap(value_parser = value_parser!(u32).range(0..=(MAIN_INDEX_MAX as _)))]
				mainIndex: u32,
			}
			let Args { mainIndex } = Args::parse();
			mainIndex << MAIN_INDEX_OFFSET
		},
		&mut toml::from_str(&io_readToString(io::stdin()).unwrap()).unwrap(),
	);
	let &mut ds1::RootStruct { xMax, yMax, numWallLayers, ref mut layers, .. } = ds1RootStruct;
	{
		let (mut i, floorLayer, [columns, rows]) = (
			0,
			&mut layers[(numWallLayers * 2) as usize],
			&mut array::from_fn(|i| vec![[i32::MAX, i32::MIN]; ([xMax, yMax][i] + 1) as _].into_boxed_slice()),
		);
		for y in 0..=yMax {
			for x in 0..=xMax {
				if floorLayer[i] & LAYER_DRAWING_PRIORITY_MASK != 0 && ![[0, 0], [xMax, yMax]].contains(&[x, y]) {
					const FLOOR_START: usize = 0;
					const FLOOR_END: usize = 1;
					{
						let column = &mut columns[x as usize];
						column[FLOOR_START].minAssign(y);
						column[FLOOR_END].maxAssign(y);
					}
					{
						let row = &mut rows[y as usize];
						row[FLOOR_START].minAssign(x);
						row[FLOOR_END].maxAssign(x);
					}
				}
				i += 1;
			}
		}
		i = 0;
		for y in 0..=yMax {
			for x in 0..=xMax {
				let cell = &mut floorLayer[i];
				*cell = {
					let &mut cell = cell;
					let settenCell = |subIndex| {
						cell & !(MAIN_INDEX_MASK | SUB_INDEX_MASK)
							| offsettedMainIndex
							| (subIndex as u32) << SUB_INDEX_OFFSET
					};
					match [columns[x as usize].contains(&y), rows[y as usize].contains(&x)] {
						[true, false] => settenCell(x),
						[false, true] => settenCell(y),
						_ => cell,
					}
				};
				i += 1;
			}
		}
	}
	stdoutRaw()
		.write_all(&toml_toStringPretty(ds1RootStruct).unwrap_or_else(|err| panic!("{err}")).into_bytes())
		.unwrap();
}
