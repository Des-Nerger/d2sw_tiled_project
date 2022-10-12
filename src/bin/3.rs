#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::PAL_LEN,
	std::{env, fs::File, io::Read},
};

fn main() {
	let mut args = env::args().skip(1);
	let swappedPAL = &mut Vec::<u8>::with_capacity(PAL_LEN);
	{
		let path: &str = &(args.next().unwrap());
		let mut file = File::open(path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
		file.read_to_end(swappedPAL).unwrap();
		assert_eq!(swappedPAL.len(), PAL_LEN);
	}
	#[allow(unused_variables)]
	let swappedPAL: &_ = swappedPAL;

	for _path in args {
		let _imageData = [98_u8; 256 * 256];
	}
}
