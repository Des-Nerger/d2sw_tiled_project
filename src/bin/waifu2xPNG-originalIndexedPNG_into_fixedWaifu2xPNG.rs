#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	array_macro::array,
	core::str::{self, FromStr},
	d2sw_tiled_project::{stdoutRaw, Image, VecExt, FULLY_TRANSPARENT},
	glam::{IVec2, IVec3, IVec4},
	memchr::memchr,
	png::ColorType,
	std::io::{self, BufRead, BufWriter, Read},
};

fn main() {
	let stdin = &mut io::stdin().lock();
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
		(vec, png.info().width as i32)
	};
	let png = &mut png::Decoder::new(stdin).read_info().unwrap();
	let ([origWidth, origHeight], ref mut orig) = {
		let image = Image::fromPNG(png);
		([image.width as i32, image.height as _], image.data)
	};
	assert_eq!(origWidth * 2, width);
	const RGB_SIZE: usize = 3;
	const RGBA_SIZE: usize = RGB_SIZE + 1;
	let (pngPAL, fixedWaifu2x) =
		(png.info().palette.as_ref().unwrap().as_ref(), &mut Vec::withLen(waifu2x.len() / RGBA_SIZE));
	{
		let (mut i, neighborhood, mut js) = (
			IVec2::ZERO,
			array![i => { let i = i as i32 - 2; IVec2::new(i % 2, i / 2) }; 5],
			// array![i => IVec2::new((i % 3) as _, (i / 3) as _) - 1; 3_usize.pow(2)],
			array![i => IVec2::new((i % 2) as _, (i / 2) as _); 2_usize.pow(2)],
		);
		while i.y < origHeight {
			let neighbors = neighborhood.map(|Δi| {
				let i = i + Δi;
				(i, {
					let iIndex = (i.x + origWidth * i.y) as usize;
					(if !(0..orig.len()).contains(&iIndex) { FULLY_TRANSPARENT } else { orig[iIndex] }) as usize
				})
			});
			for j in js {
				let jIndex = (j.x + width * j.y) as usize;
				let rgba = IVec4::from_array(
					unsafe {
						<[u8; RGBA_SIZE]>::try_from(&waifu2x[jIndex * RGBA_SIZE..][..RGBA_SIZE]).unwrap_unchecked()
					}
					.map(|colorComponent| colorComponent as _),
				);
				let (mut nearestSquaredDistance, mut nearestPALEntry) = (i32::MAX, {
					const INVALID_PAL_ENTRY: usize = usize::MAX;
					INVALID_PAL_ENTRY
				});
				for neighbor in neighbors.iter().filter_map(|&(i, neighbor)| {
					const PIXELCENTER_MUL_2: i32 = 1;
					((i * 4 - j * 2 + PIXELCENTER_MUL_2).lengthSquared() <= IVec2::new(3, 1).lengthSquared())
						.then_some(neighbor)
				}) {
					let squaredDistance = (rgba
						- IVec4::from((
							IVec3::from_array(
								unsafe {
									<[u8; RGB_SIZE]>::try_from(&pngPAL[neighbor * RGB_SIZE..][..RGB_SIZE]).unwrap_unchecked()
								}
								.map(|colorComponent| colorComponent as _),
							),
							(if neighbor == FULLY_TRANSPARENT as _ { u8::MIN } else { u8::MAX }) as _,
						)))
					.lengthSquared();
					if squaredDistance < nearestSquaredDistance {
						(nearestSquaredDistance, nearestPALEntry) = (squaredDistance, neighbor);
					}
				}
				fixedWaifu2x[jIndex] = nearestPALEntry as _;
			}
			i.x += 1;
			let Δj = {
				const ΔJ_STEP: i32 = 2;
				IVec2::from_array(if i.x == origWidth {
					i = IVec2::new(0, i.y + 1);
					[-width, 0].map(|coord| coord + ΔJ_STEP)
				} else {
					[ΔJ_STEP, 0]
				})
			};
			js = js.map(|j| j + Δj);
		}
		trait DotExt {
			fn lengthSquared(self) -> i32;
		}
		macro_rules! impl_DotExt_for {
			($ty: ty) => {
				impl DotExt for $ty {
					#[inline(always)]
					fn lengthSquared(self) -> i32 {
						self.dot(self)
					}
				}
			};
		}
		macro_rules! applyMacro {
			($ident: ident; $head: tt $(, $tail: tt )* $(,)?) => {
				$ident! $head;
				applyMacro!($ident; $( $tail ),*);
			};
			($ident: ident; ) => {};
		}
		applyMacro!(impl_DotExt_for; (IVec2), (IVec4));
	}
	let mut png = png::Encoder::new(BufWriter::new(stdoutRaw()), width as _, (origHeight * 2) as _);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(fixedWaifu2x).unwrap();
}
