#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::{unbuffered_stdout, PAL_LEN},
	std::io::{self, Read, Write},
};

fn main() {
	let pal = &mut Vec::<u8>::with_capacity(PAL_LEN);
	io::stdin().read_to_end(pal).unwrap();
	assert_eq!(pal.len(), PAL_LEN);
	for i in (0..pal.len()).step_by(3) {
		pal.swap(i + 0, i + 2);
	}
	unbuffered_stdout().write_all(pal).unwrap();
}
