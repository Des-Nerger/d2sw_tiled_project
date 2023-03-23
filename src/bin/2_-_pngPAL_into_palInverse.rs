#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::{
		iter::{self, Map},
		slice::Chunks,
	},
	d2sw_tiled_project::{stdoutRaw, DotExt, PAL_LEN, RGBCUBE_VOLUME, RGB_SIZE},
	glam::IVec3,
	rand::{thread_rng, Rng},
	std::io::{self, Read, Write},
};

fn main() {
	let (mut rng, pngPAL) = (thread_rng(), &mut Vec::<u8>::with_capacity(PAL_LEN));
	io::stdin().read_to_end(pngPAL).unwrap();
	assert_eq!(pngPAL.len(), PAL_LEN);

	#[inline(always)]
	fn ivec3Iter(palBytes: &[u8]) -> Map<Chunks<'_, u8>, fn(&[u8]) -> IVec3> {
		palBytes.chunks(3).map(|slice| -> IVec3 {
			IVec3::from_array(unsafe {
				<[u8; 3]>::try_from(slice).unwrap_unchecked().map(|colorComponent| colorComponent as _)
			})
		})
	}

	let (mut palInverse, mut squaredDistBuf) = (
		Box::<[u8; RGBCUBE_VOLUME]>::try_from(vec![rng.gen::<u8>(); RGBCUBE_VOLUME].into_boxed_slice())
			.unwrap(),
		Box::<[i32; RGBCUBE_VOLUME]>::try_from(vec![i32::MAX; RGBCUBE_VOLUME].into_boxed_slice()).unwrap(),
	);
	// Based on S. W. Thomas' "Efficient Inverse Color Map Computation" (1991),
	//   the "simple" version, that one with the complexity of O(PAL_LEN * 2.pow(3 * u8::BITS)) (in my case).
	for (i, palColor) in ivec3Iter(pngPAL).enumerate() {
		const X: i32 = 1;
		const XSQR: i32 = X.pow(2);
		const TXSQR: usize = (2 * XSQR) as _;
		let (mut rdist, [rinc, ginc, binc], mut j) =
			(palColor.lengthSquared(), (XSQR - 2 * X * palColor).to_array(), 0);
		for (rxx, _) in iter::zip((rinc..).step_by(TXSQR), 0..=u8::MAX) {
			let mut gdist = rdist;
			for (gxx, _) in iter::zip((ginc..).step_by(TXSQR), 0..=u8::MAX) {
				let mut bdist = gdist;
				for (bxx, _) in iter::zip((binc..).step_by(TXSQR), 0..=u8::MAX) {
					if squaredDistBuf[j] > bdist {
						squaredDistBuf[j] = bdist;
						palInverse[j] = i as _;
					}
					j += 1;
					bdist += bxx;
				}
				gdist += gxx;
			}
			rdist += rxx;
		}
	}

	let (mut nearestSquaredDist, mut nearestI, testColor) = (
		i32::MAX,
		rng.gen::<u8>(),
		IVec3::from_array(rng.gen::<[u8; RGB_SIZE]>().map(|colorComponent| colorComponent as _)),
	);
	eprintln!("testColor = {testColor}");
	for (i, palColor) in ivec3Iter(pngPAL).enumerate() {
		let squaredDist = (testColor - palColor).lengthSquared();
		if squaredDist < nearestSquaredDist {
			nearestSquaredDist = squaredDist;
			nearestI = i as _;
		}
	}
	let j = ((testColor.x << (2 * u8::BITS)) | (testColor.y << u8::BITS) | testColor.z) as usize;
	assert_eq!((palInverse[j], squaredDistBuf[j]), (nearestI, nearestSquaredDist));

	stdoutRaw().write_all(&palInverse[..]).unwrap();
}
