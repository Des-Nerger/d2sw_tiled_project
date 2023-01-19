#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	array_macro::array,
	core::str::{self, FromStr},
	d2sw_tiled_project::{stdoutRaw, VecExt, FULLY_TRANSPARENT},
	glam::{IVec3, IVec4},
	memchr::memchr,
	png::ColorType,
	std::io::{self, BufRead, BufWriter, Read},
};

fn main() {
	let stdin = io::stdin();
	let (stdin, stdout) = (&mut stdin.lock(), &mut BufWriter::new(stdoutRaw()));
	let (ref waifu2x, width) = {
		let (filesizeLine_len, filesize) = {
			let buffer = stdin.fill_buf().unwrap();
			let filesizeLine = str::from_utf8(&buffer[..=memchr(b'\n', buffer).unwrap()]).unwrap();
			(filesizeLine.len(), u64::from_str(filesizeLine.trim_end_matches(['\n', '\r'])).unwrap())
		};
		stdin.consume(filesizeLine_len);
		let png = &mut png::Decoder::new(stdin.take(filesize)).read_info().unwrap();
		let mut vec = Vec::withLen(png.output_buffer_size());
		let len = png.next_frame(&mut vec).unwrap().buffer_size();
		vec.setLen(len);
		(vec, png.info().width as usize)
	};
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let orig = &mut Vec::withLen(png.output_buffer_size());
	{
		let len = png.next_frame(orig).unwrap().buffer_size();
		orig.setLen(len);
	}
	let origInfo = png.info();
	assert_eq!(origInfo.width * 2, width as _);
	const RGB_SIZE: usize = 3;
	const RGBA_SIZE: usize = RGB_SIZE + 1;
	let (pngPAL, fixedWaifu2x) =
		(origInfo.palette.as_ref().unwrap().as_ref(), &mut Vec::withLen(waifu2x.len() / RGBA_SIZE));
	{
		let (mut i, mut x, neighborhood3x3, mut js) = (
			0,
			0,
			array![i => (i / 3 - 1) * origInfo.width as usize + i % 3 - 1; 3*3],
			array![i => i / 2 * width + i % 2; 2*2],
		);
		while i < orig.len() {
			let neighbors = neighborhood3x3.map(|Δi| {
				let i = i + Δi;
				(if !(0..orig.len()).contains(&i) { 0 } else { orig[i] }) as usize
			});
			for j in js {
				let rgba = IVec4::from_array(
					unsafe {
						<[u8; RGBA_SIZE]>::try_from(&waifu2x[j * RGBA_SIZE..][..RGBA_SIZE]).unwrap_unchecked()
					}
					.map(|colorComponent| colorComponent as _),
				);
				let (mut nearestDistance, mut nearestPALEntry) = {
					const INVALID_PAL_ENTRY: usize = usize::MAX;
					(i32::MAX, INVALID_PAL_ENTRY)
				};
				for neighbor in neighbors {
					let distance = (rgba
						- IVec4::from((
							IVec3::from_array(
								unsafe {
									<[u8; RGB_SIZE]>::try_from(&pngPAL[neighbor * RGB_SIZE..][..RGB_SIZE]).unwrap_unchecked()
								}
								.map(|colorComponent| colorComponent as _),
							),
							(if neighbor == FULLY_TRANSPARENT as _ { u8::MIN } else { u8::MAX }) as _,
						)))
					.abs()
					.to_array()
					.iter()
					.sum::<i32>();
					if distance < nearestDistance {
						(nearestDistance, nearestPALEntry) = (distance, neighbor);
					}
				}
				fixedWaifu2x[j] = nearestPALEntry as _;
			}
			i += 1;
			x += 2;
			let Δj = 2
				+ if x != width {
					0
				} else {
					x = 0;
					width
				};
			js = js.map(|j| j + Δj);
		}
	}
	let mut png = png::Encoder::new(stdout, width as _, origInfo.height * 2);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(fixedWaifu2x).unwrap();
}
