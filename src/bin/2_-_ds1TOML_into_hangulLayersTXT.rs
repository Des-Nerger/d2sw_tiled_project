#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::cmp::min,
	d2sw_tiled_project::{
		ds1::{
			self, existsTagLayer, LAYER_DRAWING_PRIORITY_MASK, MAIN_INDEX_MAX, MAIN_INDEX_OFFSET,
			ONE_SHADOW_LAYER, ORIENTATION_MASK, SUB_INDEX_MAX, SUB_INDEX_OFFSET,
		},
		dt1::FLOOR_ORIENTATION,
		io_readToString,
	},
	std::io,
};

fn main() {
	let ds1::RootStruct { xMax, tagType, numWallLayers, layers, .. } =
		toml::from_str(&io_readToString(io::stdin()).unwrap()).unwrap();
	for i in 0..(layers.len() - ONE_SHADOW_LAYER - existsTagLayer(tagType) as usize) {
		let isWallLayer = i < numWallLayers as usize * 2;
		if isWallLayer && i % 2 == 1 {
			continue;
		}
		let (mut j, layer) = (0, &layers[i]);
		while j < layer.len() {
			for _ in 0..=xMax {
				let cell = layer[j];
				print!(
					"{}",
					if cell & LAYER_DRAWING_PRIORITY_MASK == 0 {
						'ㅤ'
					} else {
						const HANGUL_INITIAL_MULTIPLIER: u32 = (HANGUL_MEDIAL_MAX + 1) * HANGUL_MEDIAL_MULTIPLIER;
						const HANGUL_MEDIAL_MAX: u32 = 20;
						const HANGUL_MEDIAL_MULTIPLIER: u32 = 28;
						const HANGUL_FINAL_MULTIPLIER: u32 = 1;
						const HANGUL_UNICODE_BLOCK_START: u32 = '가' as _;
						char::from_u32(
							(if isWallLayer { layers[i + 1][j] & ORIENTATION_MASK } else { FLOOR_ORIENTATION as _ })
								* HANGUL_INITIAL_MULTIPLIER
								+ min(cell >> MAIN_INDEX_OFFSET & MAIN_INDEX_MAX, HANGUL_MEDIAL_MAX)
									* HANGUL_MEDIAL_MULTIPLIER
								+ (cell >> SUB_INDEX_OFFSET & SUB_INDEX_MAX) * HANGUL_FINAL_MULTIPLIER
								+ HANGUL_UNICODE_BLOCK_START,
						)
						.unwrap()
					}
				);
				j += 1;
			}
			println!(",");
		}
		println!();
	}
}
