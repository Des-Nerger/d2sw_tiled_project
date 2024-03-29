#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::iter,
	d2sw_tiled_project::{
		stdoutRaw, unlet, VecExt, BLACK, FULLY_TRANSPARENT, PAL_LEN, RGBA_SIZE, RGBCUBE_VOLUME,
	},
	png::ColorType,
	std::io::{self, BufWriter, Read},
};

fn main() {
	let stdin = &mut io::stdin().lock();
	let boxedSlice = {
		let mut vec = Vec::<u8>::with_capacity(PAL_LEN + RGBCUBE_VOLUME);
		stdin.take(vec.capacity() as _).read_to_end(&mut vec).unwrap();
		vec.into_boxed_slice()
	};
	let pngPAL;
	let palInverse = {
		let palInverse;
		(pngPAL, palInverse) = boxedSlice.split_at(PAL_LEN);
		<&[_; RGBCUBE_VOLUME]>::try_from(palInverse).unwrap()
	};
	unlet!(boxedSlice);
	let (rgbaData, width, height) = {
		let png = &mut png::Decoder::new(stdin).read_info().unwrap();
		let mut vec = Vec::withLen(png.output_buffer_size());
		let len = png.next_frame(&mut vec).unwrap().buffer_size();
		vec.setLen(len);
		let &png::Info { width, height, .. } = png.info();
		(vec, width, height)
	};
	let mut indexedColor_data = Vec::withLen((width * height) as _);
	for (indexedColor_pixel, [red, green, blue, alpha]) in iter::zip(
		indexedColor_data.iter_mut(),
		rgbaData.as_slice().chunks(RGBA_SIZE).map(|slice| {
			unsafe { <[u8; RGBA_SIZE]>::try_from(slice.get_unchecked(..RGBA_SIZE)).unwrap_unchecked() }
				.map(|colorComponent| colorComponent as usize)
		}),
	) {
		*indexedColor_pixel = if alpha <= (u8::MAX / 16) as _ {
			FULLY_TRANSPARENT
		} else {
			match palInverse[(red << (2 * u8::BITS)) | (green << u8::BITS) | blue] {
				0 => BLACK,
				nonZero => nonZero,
			}
		}
	}
	let mut png = png::Encoder::new(BufWriter::new(stdoutRaw()), width, height);
	png.set_color(ColorType::Indexed);
	png.set_palette(pngPAL);
	png.set_trns(&[0][..]);
	png.write_header().unwrap().write_image_data(&indexedColor_data).unwrap();
}
