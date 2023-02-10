#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{ds1, io_readToString, stdoutRaw},
	std::io::{self, BufWriter},
};

fn main() {
	toml::from_str::<ds1::RootStruct>(&io_readToString(io::stdin()).unwrap())
		.unwrap()
		.writeTo(&mut BufWriter::new(stdoutRaw()));
}
