#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

pub const PAL_LEN: usize = 256 * 3;

pub mod dt1 {
	use {
		byteorder::{ReadBytesExt, LE},
		core::{fmt, ops},
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

	const EXPECTED_VERSION: [i32; 2] = [7, 6];

	pub struct VersionMismatchError {
		version: [i32; 2],
	}
	impl fmt::Debug for VersionMismatchError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "dt1.fileHeader.version == {:?} != {EXPECTED_VERSION:?}", self.version)
		}
	}

	const SUBTILE_SIZE: usize = 5;
	const NUM_SUBTILES: usize = SUBTILE_SIZE.pow(2);

	#[derive(Serialize, Deserialize)]
	pub struct Tile {
		pub direction: i32,
		pub roofHeight: i16,
		pub materialFlags: [u8; 2],
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
		pub usuallyZeros: [u8; 4],

		#[serde(rename = "block")]
		pub blocks: Vec<Block>,
	}

	impl Tile {
		pub fn height(&self) -> usize {
			80
		}
	}

	#[derive(Serialize, Deserialize)]
	pub struct Block {
		pub x: i16,
		pub y: i16,
		pub gridX: u8,
		pub gridY: u8,
		pub format: [u8; 2],
		pub length: i32,
		pub fileOffset: i32,
	}

	impl Metadata {
		pub fn new(dt1: &[u8]) -> Result<Metadata, VersionMismatchError> {
			let mut cursor = io::Cursor::new(dt1);
			let version = [cursor.read_i32::<LE>().unwrap(), cursor.read_i32::<LE>().unwrap()];
			if version != EXPECTED_VERSION {
				return Err(VersionMismatchError { version });
			}
			cursor.consumeZeros(260);
			let mut tiles = Vec::with_capacity(cursor.read_i32::<LE>().unwrap() as _);
			let tileHeadersPointer = cursor.read_i32::<LE>().unwrap();
			assert_eq!(cursor.position(), tileHeadersPointer as _);
			for _ in 0..tiles.capacity() {
				tiles.push(Tile {
					direction: cursor.read_i32::<LE>().unwrap(),
					roofHeight: cursor.read_i16::<LE>().unwrap(),
					materialFlags: cursor.read_u8_array(),
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
					usuallyZeros: {
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
						format: cursor.read_u8_array(),
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

			trait ReadExt {
				fn consumeZeros(&mut self, zerosCount: usize);
				fn read_u8_array<const N: usize>(&mut self) -> [u8; N];
			}
			impl ReadExt for io::Cursor<&[u8]> {
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
			trait Copy_AddAssign_Ext {
				fn alsoAddTo(self, to: &mut Self) -> Self;
			}
			impl<T: Copy + ops::AddAssign> Copy_AddAssign_Ext for T {
				fn alsoAddTo(self, to: &mut Self) -> Self {
					*to += self;
					self
				}
			}

			Ok(Metadata { fileHeader: FileHeader { version, tileHeadersPointer }, tiles })
		}
	}

	pub trait DrawDestination {
		fn widthLog2(&self) -> usize;
		fn putpixel(&mut self, atIndex: usize, withValue: u8);

		#[inline(always)]
		fn width(&self) -> usize {
			1 << self.widthLog2()
		}

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
			let (mut i, mut y, widthLog2) = (0, 0, self.widthLog2());
			while length > 0 {
				static XJUMP: [u8; 15] = [14, 12, 10, 8, 6, 4, 2, 0, 2, 4, 6, 8, 10, 12, 14];
				static NBPIX: [u8; 15] = [4, 8, 12, 16, 20, 24, 28, 32, 28, 24, 20, 16, 12, 8, 4];
				let (mut j, mut n) = (((y0 + y) << widthLog2) + x0 + XJUMP[y] as usize, NBPIX[y] as usize);
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
			let (mut length, widthLog2) = (data.len(), self.widthLog2());

			// draw
			let (mut i, mut y, j0) = (0, 0, |y| ((y0 + y) << widthLog2) + x0);
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

#[derive(Debug, Copy, Clone)]
pub struct TileColumns {
	pub fullColumnHeight: usize,
	pub numFullColumns: usize,
	pub lastColumnHeight: usize,
}

impl TileColumns {
	#[inline(always)]
	pub fn pushTile(&mut self, tileHeight: usize) -> usize {
		self.lastColumnHeight += tileHeight;
		let diff = self.lastColumnHeight - self.fullColumnHeight;
		if (diff as isize) < 0 {
			return 0;
		}
		self.numFullColumns += 1;
		(|(lastColumnHeight, fillerHeight)| {
			self.lastColumnHeight = lastColumnHeight;
			fillerHeight
		})(if diff == 0 { (0, 0) } else { (tileHeight, tileHeight - diff) })
	}
}

/*
pub struct TileColumns {
	pub columnHeight: usize,
	totalHeight: usize,
}

impl TileColumns {
	#[inline(always)]
	pub fn pushTile(&mut self, tileHeight: usize) -> usize {
		self.totalHeight += tileHeight;
		let diff = tileHeight - self.totalHeight.modCeil(self.columnHeight);
		if diff as isize > 0 {
			self.totalHeight += diff;
			diff
		} else {
			0
		}
	}
}

pub struct TilesSquare<const TILEWIDTH: usize> {
	pub sizeLog2: usize,
	pub usedHeight: usize,
}

impl<const TILEWIDTH: usize> TilesSquare<TILEWIDTH> {
	#[inline(always)]
	pub fn putTile(&mut self, tileHeight: usize) -> usize {
		let square = self;
		let (size, mut usedHeight) = (1 << square.sizeLog2, square.usedHeight + tileHeight);
		let excess = {
			let diff = tileHeight - usedHeight.bitandCeil(size - 1);
			if diff as isize > 0 {
				usedHeight += diff;
				diff
			} else {
				0
			}
		};
		if (usedHeight.shrCeil(square.sizeLog2) * TILEWIDTH) > size {
			tileHeight
		} else {
			square.usedHeight = usedHeight;
			excess
		}
	}
}
*/

pub trait NonZeroIntegerExt {
	fn nextShlOf(self, rhs: Self) -> Self;
	fn shrCeil(self, rhs: Self) -> Self;
	fn bitandCeil(self, rhs: Self) -> Self;
}
impl NonZeroIntegerExt for usize {
	#[inline(always)]
	fn nextShlOf(self, rhs: Self) -> Self {
		let rhsExp2 = 1 << rhs;
		let r = self & (rhsExp2 - 1);
		self + ((!((r != 0) as Self) + 1) & (rhsExp2 - r))
	}
	#[inline(always)]
	fn shrCeil(self, rhs: Self) -> Self {
		((self - 1) >> rhs) + 1
	}
	#[inline(always)]
	fn bitandCeil(self, rhs: Self) -> Self {
		((self - 1) & rhs) + 1
	}
}

#[inline(always)]
pub const fn log2(of: usize) -> usize {
	(usize::BITS - 1 - of.leading_zeros()) as _
}
#[inline(always)]
pub const fn log2Ceil(of: usize) -> usize {
	log2(of - 1) + 1
}
#[macro_export]
macro_rules! log2 {
	( $of:expr ) => {{
		const LOG2: usize = log2($of);
		LOG2
	}};
}
#[macro_export]
macro_rules! log2Ceil {
	( $of:expr ) => {{
		const LOG2CEIL: usize = log2Ceil($of);
		LOG2CEIL
	}};
}

#[macro_export]
macro_rules! array_fromFn {
	(|$i: ident| $expr: expr) => {{
		#[inline(always)]
		const fn array_fromFn() -> [T; N] {
			let mut array: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
			{
				let mut i = 0;
				while i < array.len() {
					array[i] = {
						let $i = i;
						MaybeUninit::new($expr as T)
					};
					i += 1;
				}
			}
			unsafe { transmute::<_, [T; N]>(array) }
		}
		array_fromFn()
	}};
}

use std::{fs::File, os};

#[cfg(unix)]
pub fn stdoutRaw() -> File {
	use os::unix::io::FromRawFd;
	unsafe { File::from_raw_fd(1) }
}

#[cfg(windows)]
pub fn stdoutRaw() -> File {
	use windows::io::{AsRawHandle, FromRawHandle};
	unsafe { File::from_raw_handle(io::stdout().as_raw_handle()) }
}
