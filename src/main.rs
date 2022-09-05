#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	byteorder::{ReadBytesExt, LE},
	std::{
		fs::File,
		io::{self, Read},
	},
};

fn main() {
	let path = "global/tiles/ACT1/Town/Floor.dt1";
	let buffer = &mut Vec::<u8>::new();
	{
		let mut file = File::open(path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
		file.read_to_end(buffer).unwrap();
	}
	let mut cursor = io::Cursor::new(buffer.as_slice());
	eprintln!("[fileHeader]");
	macro_rules! stringifyId {
		($id: ident) => {{
			_ = $id;
			stringify!($id)
		}};
	}
	let version = [cursor.read_i32::<LE>().unwrap(), cursor.read_i32::<LE>().unwrap()];
	eprintln!("{} = {version:?}", stringifyId!(version));
	fn allZeros(byteSlice: &[u8]) -> bool {
		for &byte in byteSlice {
			if byte != 0 {
				return false;
			}
		}
		true
	}
	trait Trait {
		fn consumeZeros(&mut self, zerosCount: usize);
	}
	impl Trait for io::Cursor<&[u8]> {
		#[inline]
		fn consumeZeros(&mut self, zerosCount: usize) {
			let position = self.position() as usize;
			let newPosition = position + zerosCount;
			let underlyingBuffer = *(self.get_ref());
			assert!(allZeros(&underlyingBuffer[position..newPosition]));
			self.set_position(newPosition as _);
		}
	}
	cursor.consumeZeros(260);
	let numTiles = cursor.read_i32::<LE>().unwrap();
	let tileHeadersPointer = cursor.read_i32::<LE>().unwrap();
	eprint!(
		"{} = {numTiles}\n{} = {tileHeadersPointer}\n",
		stringifyId!(numTiles),
		stringifyId!(tileHeadersPointer)
	);
	for i in 0..numTiles {
		let direction = cursor.read_i32::<LE>().unwrap();
		let roofHeight = cursor.read_i16::<LE>().unwrap();
		let soundIndex = cursor.read_i8().unwrap();
		let isAnimated = cursor.read_i8().unwrap();
		let height = cursor.read_i32::<LE>().unwrap();
		let width = cursor.read_i32::<LE>().unwrap();
		eprintln!(
			"\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
			format_args!("[[tileHeader]]"),
			format_args!("{} = {direction}", stringifyId!(direction)),
			format_args!("{} = {roofHeight}", stringifyId!(roofHeight)),
			format_args!("{} = {soundIndex}", stringifyId!(soundIndex)),
			format_args!("{} = {isAnimated}", stringifyId!(isAnimated)),
			format_args!("{} = {height}", stringifyId!(height)),
			format_args!("{} = {width}", stringifyId!(width)),
		);
		cursor.consumeZeros(4);
		let orientation = cursor.read_i32::<LE>().unwrap();
		let mainIndex = cursor.read_i32::<LE>().unwrap();
		let subIndex = cursor.read_i32::<LE>().unwrap();
		let rarity = cursor.read_i32::<LE>().unwrap();
		let unknown = [cursor.read_i16::<LE>().unwrap(), cursor.read_i16::<LE>().unwrap()];
		const SUBTILE_SIZE: usize = 5;
		const NUM_SUBTILES: usize = SUBTILE_SIZE.pow(2);
		let mut flags = [-1_i8; NUM_SUBTILES];
		for flag in flags.iter_mut() {
			*flag = cursor.read_i8().unwrap();
		}
		cursor.consumeZeros(7);
		let blockHeadersPointer = cursor.read_i32::<LE>().unwrap();
		let blockDataLength = cursor.read_i32::<LE>().unwrap();
		let numBlocks = cursor.read_i32::<LE>().unwrap();
		eprintln!(
			"id = {{{}, {}, {}}}\n{}\n{}\n{}\n{}\n{}\n{}",
			format_args!("{} = {orientation}", stringifyId!(orientation)),
			format_args!("{} = {mainIndex}", stringifyId!(mainIndex)),
			format_args!("{} = {subIndex}", stringifyId!(subIndex)),
			format_args!("{} = {rarity}", stringifyId!(rarity)),
			format_args!("{} = {unknown:?}", stringifyId!(unknown)),
			format_args!("{} = {flags:#?}", stringifyId!(flags)),
			format_args!("{} = {blockHeadersPointer}", stringifyId!(blockHeadersPointer)),
			format_args!("{} = {blockDataLength}", stringifyId!(blockDataLength)),
			format_args!("{} = {numBlocks}", stringifyId!(numBlocks)),
		);
		cursor.consumeZeros(12); // they're not all zeros all of the time; FIXME later
		if i == 1 {
			break;
		}
	}
}
