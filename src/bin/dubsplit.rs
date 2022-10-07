#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	const_format::concatcp,
	core::{cmp::min, str::FromStr},
	d2sw_tiled_project::unbuffered_stdout,
	std::{
		env,
		fs::File,
		io::{self, BufRead, Write},
	},
};

fn main() {
	const FILESIZE_MAX: usize = usize::MAX;
	const FILESIZE_LINE: &'static str = concatcp!(FILESIZE_MAX, '\n');
	let (stdin, filesizeLine) = (io::stdin(), &mut String::with_capacity(FILESIZE_LINE.len()));
	let (mut stdin, mut envArgs) = (stdin.lock(), env::args().skip(1));
	'outer: loop {
		let (mut file, mut filesize) = if let Some(filepath) = envArgs.next() {
			filesizeLine.clear();
			stdin.read_line(filesizeLine).unwrap();
			(
				File::create(filepath).unwrap(),
				usize::from_str(filesizeLine.trim_end_matches(&['\n', '\r'])).unwrap(),
			)
		} else {
			(unbuffered_stdout(), FILESIZE_MAX)
		};
		while filesize > 0 {
			let buffer = stdin.fill_buf().unwrap();
			if buffer.len() == 0 {
				break 'outer;
			}
			let len = min(filesize, buffer.len());
			file.write_all(&buffer[..len]).unwrap();
			stdin.consume(len);
			filesize -= len;
		}
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
}
