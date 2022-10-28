#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	const_format::formatcp,
	core::str::FromStr,
	d2sw_tiled_project::{
		dt1::{self, DrawDestination},
		stdoutRaw,
	},
	std::io::{self, BufRead, Read, Write},
};

fn main() {
	let (stdin, stdout) = (io::stdin(), &mut io::BufWriter::new(stdoutRaw()));
	type Filesize = usize;
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
	let (stdin, filesizeLine, dt1, counts) = (
		&mut stdin.lock(),
		&mut String::with_capacity(FILESIZE_LINE.len()),
		&mut Vec::new(),
		&mut [0; UsedPALIndicesFrequency::LEN],
	);
	'outer: while {
		filesizeLine.clear();
		stdin.read_line(filesizeLine).unwrap() != 0
	} {
		for tile in {
			let filesize = Filesize::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap();
			dt1.clear();
			dt1.reserve(filesize);
			dt1.setLen(filesize);
			stdin.read_exact(dt1).unwrap();

			trait VecExt {
				fn setLen(&mut self, newLen: usize);
			}
			impl<T> VecExt for Vec<T> {
				fn setLen(&mut self, newLen: usize) {
					if !cfg!(debug_assertions) {
						assert!(newLen <= self.capacity());
					}
					unsafe { self.set_len(newLen) };
				}
			}

			match dt1::Metadata::new(dt1) {
				Err(_) => continue 'outer,
				Ok(ok) => ok,
			}
		}
		.tiles
		{
			for block in &tile.blocks {
				(if block.format == [1, 0] {
					UsedPALIndicesFrequency::drawBlockIsometric
				} else {
					UsedPALIndicesFrequency::drawBlockNormal
				})(
					&mut UsedPALIndicesFrequency(counts),
					IRRELEVANT,
					IRRELEVANT,
					&dt1[(tile.blockHeadersPointer + block.fileOffset) as _..][..block.length as _],
				);
			}
		}
	}
	let mut arrayIndices: [u8; UsedPALIndicesFrequency::LEN] =
		(0..=(UsedPALIndicesFrequency::LEN - 1) as u8).collect::<Vec<_>>().try_into().unwrap();
	arrayIndices.sort_by(|a, b| counts[*a as usize].cmp(&counts[*b as usize]));
	for i in arrayIndices {
		writeln!(stdout, "{i}\t{}", counts[i as usize]).unwrap();
	}

	struct UsedPALIndicesFrequency<'a>(&'a mut [usize; UsedPALIndicesFrequency::LEN]);
	impl UsedPALIndicesFrequency<'_> {
		const LEN: usize = u8::MAX as usize + 1;
	}
	const IRRELEVANT: usize = 0;
	impl DrawDestination for UsedPALIndicesFrequency<'_> {
		#[inline(always)]
		fn widthLog2(&self) -> usize {
			IRRELEVANT
		}
		#[inline(always)]
		fn putpixel(&mut self, _atIndex: usize, withValue: u8) {
			self.0[withValue as usize] += 1;
		}
	}
}
