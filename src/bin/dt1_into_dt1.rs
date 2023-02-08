#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{dt1, stdoutRaw},
	std::io::{self, BufWriter, Read},
};

fn main() -> Result<(), dt1::VersionMismatchError> {
	let dt1 = &mut Vec::<u8>::new();
	io::stdin().read_to_end(dt1).unwrap();
	dt1::Metadata::new(dt1)?.writeWithBlockDataFromDT1(dt1, &mut BufWriter::new(stdoutRaw()));
	Ok(())
}
