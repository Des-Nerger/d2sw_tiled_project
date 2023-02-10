#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	array_macro::array,
	const_format::formatcp,
	core::str::FromStr,
	d2sw_tiled_project::{
		dt1::{self, DrawDestination},
		stdoutRaw, VecExt,
	},
	std::io::{self, BufRead, Read, Write},
};

fn main() {
	type Filesize = usize;
	const FILESIZE_LINE: &'static str = formatcp!("{}\r\n", Filesize::MAX);
	let (stdin, stdout, filesizeLine, dt1, counts) = &mut (
		io::stdin().lock(),
		io::BufWriter::new(stdoutRaw()),
		String::with_capacity(FILESIZE_LINE.len()),
		Vec::new(),
		[0; UsedPALIndicesFrequency::LEN],
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
			match dt1::Metadata::new(dt1) {
				Err(_) => continue 'outer,
				Ok(ok) => ok,
			}
		}
		.tiles
		{
			for block in &tile.blocks {
				block.drawFn()(
					&mut UsedPALIndicesFrequency(counts),
					IRRELEVANT,
					IRRELEVANT,
					&dt1[(tile.blockHeadersPointer + block.fileOffset) as _..][..block.length as _],
				);
			}
		}
	}
	assert_eq!(filesizeLine.capacity(), FILESIZE_LINE.len());
	let mut arrayIndices = array![j => j as u8; UsedPALIndicesFrequency::LEN];
	arrayIndices.sort_by_key(|&i| counts[i as usize]);
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
		fn width(&self) -> usize {
			IRRELEVANT
		}
		#[inline(always)]
		fn putpixel(&mut self, _atIndex: usize, value: u8) {
			self.0[value as usize] += 1;
		}
	}
}
