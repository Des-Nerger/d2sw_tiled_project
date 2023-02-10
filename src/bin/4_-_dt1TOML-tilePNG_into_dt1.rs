#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::str::{self, FromStr},
	d2sw_tiled_project::{dt1, io_readToString, stdoutRaw, Image},
	memchr::memchr,
	std::{
		io::{self, BufRead, BufWriter, Read},
		process::ExitCode,
	},
};

fn main() -> ExitCode {
	let stdin = &mut io::stdin().lock();
	{
		let (filesizeLine_len, filesize) = {
			let buffer = stdin.fill_buf().unwrap();
			let filesizeLine = str::from_utf8(
				&buffer[..={
					match memchr(b'\n', buffer) {
						Some(index) => index,
						None => return ExitCode::FAILURE,
					}
				}],
			)
			.unwrap();
			(filesizeLine.len(), u64::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap())
		};
		stdin.consume(filesizeLine_len);
		toml::from_str::<dt1::Metadata>(&io_readToString(stdin.take(filesize)).unwrap()).unwrap()
	}
	.writeWithBlockDataFromTileImage(
		&Image::fromPNG(&mut png::Decoder::new(stdin).read_info().unwrap()),
		&mut BufWriter::new(stdoutRaw()),
	);
	ExitCode::SUCCESS
}
