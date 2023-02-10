#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	const_format::formatcp,
	core::str::FromStr,
	d2sw_tiled_project::stdoutRaw,
	std::{
		env,
		fs::File,
		io::{self, BufRead, Read},
	},
};

fn main() {
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", u64::MAX);
	let (stdin, filesizeLine) = &mut (io::stdin().lock(), String::with_capacity(FILESIZE_LINE.len()));
	for filepath in env::args().skip(1) {
		filesizeLine.clear();
		if stdin.read_line(filesizeLine).unwrap() == 0 {
			break;
		}
		io::copy(
			&mut stdin.take(u64::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap()),
			&mut File::create(filepath).unwrap(),
		)
		.unwrap();
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
	io::copy(stdin, &mut stdoutRaw()).unwrap();
}
