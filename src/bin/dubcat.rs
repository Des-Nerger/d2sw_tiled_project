#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	d2sw_tiled_project::stdoutRaw,
	std::{
		fs::File,
		io::{self, BufRead, Write},
	},
};

fn main() {
	let (stdin, stdout) = (io::stdin(), &mut io::BufWriter::new(stdoutRaw()));
	let (stdin, filepathLine) = (&mut stdin.lock(), &mut String::new());
	while {
		filepathLine.clear();
		stdin.read_line(filepathLine).unwrap() != 0
	} {
		let file = &mut {
			let filepath = filepathLine.trim_end_matches(['\n', '\r']);
			File::open(filepath).unwrap_or_else(|err| panic!("{filepath:?}: {err}"))
		};
		writeln!(stdout, "{}", file.metadata().unwrap().len()).unwrap();

		// cast BufWriter into generic Write in order to avoid unnecessary flushes
		io::copy(file, stdout as &mut dyn Write).unwrap();
	}
}
