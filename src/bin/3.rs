#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use std::{env, fs::File, io::Read};

fn main() {
	let mut args = env::args().skip(1);
	const PAL_LEN: usize = 256 * 3;
	let swappedPal = &mut Vec::<u8>::with_capacity(PAL_LEN);
	{
		let path: &str = &(args.next().unwrap());
		let mut file = File::open(path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
		file.read_to_end(swappedPal).unwrap();
		assert_eq!(swappedPal.len(), PAL_LEN);
	}
	#[allow(unused_variables)]
	let swappedPal: &_ = swappedPal;

	for _path in args {
		let _imageData = [98_u8; 256 * 256];
	}
}
