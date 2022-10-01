#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

pub mod dt1 {
	use {
		byteorder::{ReadBytesExt, LE},
		core::ops,
		serde::{Deserialize, Serialize},
		std::io::{self, BufRead},
	};

	#[derive(Serialize, Deserialize)]
	pub struct Metadata {
		pub fileHeader: FileHeader,

		#[serde(rename = "tile")]
		pub tiles: Vec<Tile>,
	}

	#[derive(Serialize, Deserialize)]
	pub struct FileHeader {
		pub version: [i32; 2],
		pub tileHeadersPointer: i32,
	}

	const SUBTILE_SIZE: usize = 5;
	const NUM_SUBTILES: usize = SUBTILE_SIZE.pow(2);

	#[derive(Serialize, Deserialize)]
	pub struct Tile {
		pub direction: i32,
		pub roofHeight: i16,
		pub soundIndex: u8,
		pub isAnimated: bool,
		pub height: i32,
		pub width: i32,
		pub orientation: i32,
		pub mainIndex: i32,
		pub subIndex: i32,
		pub rarityOrFrameIndex: i32,
		pub unknown: [u8; 4],
		pub subtileFlags: [u8; NUM_SUBTILES],
		pub blockHeadersPointer: i32,
		pub blockDataLength: i32,
		pub almostAlwaysZeros: [u8; 4],

		#[serde(rename = "block")]
		pub blocks: Vec<Block>,
	}

	#[derive(Serialize, Deserialize)]
	pub struct Block {
		pub x: i16,
		pub y: i16,
		pub gridX: u8,
		pub gridY: u8,
		pub format: i16,
		pub length: i32,
		pub fileOffset: i32,
	}

	impl Metadata {
		pub fn new(dt1: &[u8]) -> Metadata {
			let mut cursor = io::Cursor::new(dt1);
			let version = [cursor.read_i32::<LE>().unwrap(), cursor.read_i32::<LE>().unwrap()];
			cursor.consumeZeros(260);
			let mut tiles = Vec::with_capacity(cursor.read_i32::<LE>().unwrap() as _);
			let tileHeadersPointer = cursor.read_i32::<LE>().unwrap();
			assert_eq!(cursor.position(), tileHeadersPointer as _);
			for _ in 0..tiles.capacity() {
				tiles.push(Tile {
					direction: cursor.read_i32::<LE>().unwrap(),
					roofHeight: cursor.read_i16::<LE>().unwrap(),
					soundIndex: cursor.read_u8().unwrap(),
					isAnimated: match cursor.read_u8().unwrap() {
						0 => false,
						1 => true,
						byte => panic!("{}", byte),
					},
					height: cursor.read_i32::<LE>().unwrap(),
					width: cursor.read_i32::<LE>().unwrap(),
					orientation: {
						cursor.consumeZeros(4);
						cursor.read_i32::<LE>().unwrap()
					},
					mainIndex: cursor.read_i32::<LE>().unwrap(),
					subIndex: cursor.read_i32::<LE>().unwrap(),
					rarityOrFrameIndex: cursor.read_i32::<LE>().unwrap(),
					unknown: cursor.read_u8_array(),
					subtileFlags: cursor.read_u8_array(),
					blockHeadersPointer: {
						cursor.consumeZeros(7);
						cursor.read_i32::<LE>().unwrap()
					},
					blockDataLength: cursor.read_i32::<LE>().unwrap(),
					blocks: Vec::with_capacity(cursor.read_i32::<LE>().unwrap() as _),
					almostAlwaysZeros: {
						cursor.consumeZeros(4);
						cursor.read_u8_array()
					},
				});
				cursor.consumeZeros(4);
			}
			assert_eq!(tiles.len(), tiles.capacity());
			for tile in &mut tiles {
				assert_eq!(cursor.position(), tile.blockHeadersPointer as _);
				let mut totalLength = 0;
				let blocks = &mut tile.blocks;
				for _ in 0..blocks.capacity() {
					blocks.push(Block {
						x: cursor.read_i16::<LE>().unwrap(),
						y: cursor.read_i16::<LE>().unwrap(),
						gridX: {
							cursor.consumeZeros(2);
							cursor.read_u8().unwrap()
						},
						gridY: cursor.read_u8().unwrap(),
						format: cursor.read_i16::<LE>().unwrap(),
						length: cursor.read_i32::<LE>().unwrap().alsoAddTo(&mut totalLength),
						fileOffset: {
							cursor.consumeZeros(2);
							cursor.read_i32::<LE>().unwrap()
						},
					});
				}
				assert_eq!(blocks.len(), blocks.capacity());
				cursor.consume(totalLength as _);
			}
			assert_eq!(cursor.position(), dt1.len() as _);

			trait Read {
				fn consumeZeros(&mut self, zerosCount: usize);
				fn read_u8_array<const N: usize>(&mut self) -> [u8; N];
			}
			impl Read for io::Cursor<&[u8]> {
				fn consumeZeros(&mut self, zerosCount: usize) {
					let position = self.position() as usize;
					self.set_position((position + zerosCount) as _);
					let underlyingSlice = *(self.get_ref());
					assert!(allZeros(&underlyingSlice[position..self.position() as _]));

					fn allZeros(byteSlice: &[u8]) -> bool {
						for &byte in byteSlice {
							if byte != 0 {
								return false;
							}
						}
						true
					}
				}
				fn read_u8_array<const N: usize>(&mut self) -> [u8; N] {
					let position = self.position() as usize;
					self.set_position((position + N) as _);
					let underlyingSlice = *(self.get_ref());
					<[u8; N]>::try_from(&underlyingSlice[position..self.position() as _]).unwrap()
				}
			}

			#[allow(non_camel_case_types)]
			trait Copy_AddAssign {
				fn alsoAddTo(self, to: &mut Self) -> Self;
			}
			impl<T: Copy + ops::AddAssign> Copy_AddAssign for T {
				fn alsoAddTo(self, to: &mut Self) -> Self {
					*to += self;
					self
				}
			}

			Metadata { fileHeader: FileHeader { version, tileHeadersPointer }, tiles }
		}
	}

	pub trait DrawDestination {
		fn width(&self) -> usize;
		fn putpixel(&mut self, atIndex: usize, withValue: u8);

		/*
			3D-isometric Block :

			1st line : draw a line of 4 pixels
			2nd line : draw a line of 8 pixels
			3rd line : draw a line of 12 pixels
			and so on...
		*/
		fn drawBlockIsometric(&mut self, x0: usize, y0: usize, data: &[u8]) {
			let mut length = data.len();

			// 3d-isometric subtile is 256 bytes, no more, no less
			assert_eq!(length, 256);

			// draw
			let (mut i, mut y, width) = (0, 0, self.width());
			while length > 0 {
				static XJUMP: [u8; 15] = [14, 12, 10, 8, 6, 4, 2, 0, 2, 4, 6, 8, 10, 12, 14];
				static NBPIX: [u8; 15] = [4, 8, 12, 16, 20, 24, 28, 32, 28, 24, 20, 16, 12, 8, 4];
				let (mut j, mut n) = ((y0 + y) * width + x0 + XJUMP[y] as usize, NBPIX[y] as usize);
				length -= n;
				while n != 0 {
					self.putpixel(j, data[i]);
					i += 1;
					j += 1;
					n -= 1;
				}
				y += 1;
			}
		}

		/*
			RLE Block :

			1st byte is pixels to "jump", 2nd is number of "solid" pixels, followed by the pixel color indexes.
			when 1st and 2nd bytes are 0 and 0, next line.
		*/
		fn drawBlockNormal(&mut self, x0: usize, y0: usize, data: &[u8]) {
			let (mut length, width) = (data.len(), self.width());

			// draw
			let (mut i, mut y, j0) = (0, 0, |y| (y0 + y) * width + x0);
			let mut j = j0(y);
			while length > 0 {
				let (xjump, mut xsolid) = (data[i + 0] as usize, data[i + 1] as usize);
				i += 2;
				length -= 2;
				if xjump != 0 || xsolid != 0 {
					j += xjump;
					length -= xsolid;
					while xsolid != 0 {
						self.putpixel(j, data[i]);
						i += 1;
						j += 1;
						xsolid -= 1;
					}
				} else {
					y += 1;
					j = j0(y);
				}
			}
		}
	}
}
