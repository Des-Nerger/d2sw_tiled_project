#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

pub mod ds1 {
	use {
		super::{ReadExt, VecExt},
		byteorder::{ReadBytesExt, LE},
		core::{array, fmt, mem::size_of},
		memchr::memchr,
		serde::{Deserialize, Serialize},
		std::io::{self, BufRead},
	};

	#[derive(Serialize, Deserialize)]
	pub struct RootStruct {
		pub version: i32,
		pub xMax: i32,
		pub yMax: i32,
		pub actIndex: i32,
		pub tagType: i32,
		pub files: Vec<String>,
		pub unknown: Option<[u8; 2 * size_of::<i32>()]>,
		pub numWallLayers: i32,
		pub numFloors: i32,
		pub layers: Vec<Vec<u32>>,

		#[serde(rename = "object")]
		pub objects: Option<Vec<Object>>,

		#[serde(rename = "group")]
		pub groups: Option<Vec<Group>>,

		#[serde(rename = "path")]
		pub paths: Option<Vec<Path>>,
	}

	#[derive(Serialize, Deserialize)]
	pub struct Object {
		pub r#type: i32,
		pub id: i32,
		pub x: i32,
		pub y: i32,
		pub flags: i32,
	}

	#[derive(Serialize, Deserialize)]
	pub struct Group {
		pub x: i32,
		pub y: i32,
		pub width: i32,
		pub height: i32,
		pub unknown: i32,
	}

	#[derive(Serialize, Deserialize)]
	pub struct Path {
		pub x: i32,
		pub y: i32,

		#[serde(rename = "node")]
		pub nodes: Vec<Node>,
	}

	#[derive(Serialize, Deserialize)]
	pub struct Node {
		pub x: i32,
		pub y: i32,
		pub action: i32,
	}

	pub const ONE_SHADOW_LAYER: usize = 1;
	const MINIMUM_VERSION: i32 = 7;

	pub struct VersionMismatchError {
		version: i32,
	}
	impl fmt::Debug for VersionMismatchError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "ds1.version == {:?} < {MINIMUM_VERSION:?}", self.version)
		}
	}

	#[inline(always)]
	pub const fn existsTagLayer(tagType: i32) -> bool {
		matches!(tagType, 1 | 2)
	}

	impl RootStruct {
		pub fn new(cursor: &mut io::Cursor<impl AsRef<[u8]>>) -> Result<Self, VersionMismatchError> {
			let version = cursor.read_i32::<LE>().unwrap();
			if version < MINIMUM_VERSION {
				return Err(VersionMismatchError { version });
			}
			let [xMax, yMax] = array::from_fn(|_| cursor.read_i32::<LE>().unwrap());
			let actIndex = if version < 8 { 0 } else { cursor.read_i32::<LE>().unwrap() };
			let tagType = if version < 10 { 0 } else { cursor.read_i32::<LE>().unwrap() };
			let mut files = Vec::with_capacity(cursor.read_i32::<LE>().unwrap() as _);
			for _ in 0..files.capacity() {
				let ds1 = cursor.get_ref().as_ref();
				let unreadDS1 = &ds1[cursor.position() as _..];
				let nulPosition = memchr(b'\0', unreadDS1).unwrap();
				files.push(String::from_utf8((&unreadDS1[..nulPosition]).to_vec()).unwrap());
				cursor.consume(nulPosition + 1);
			}
			let unknown = if matches!(version, 9..=13) { Some(cursor.read_u8_array()) } else { None };
			let numWallLayers = cursor.read_i32::<LE>().unwrap();
			let numFloors = if version < 16 { 1 } else { cursor.read_i32::<LE>().unwrap() };
			let mut layers = Vec::new();
			for _ in 0..numWallLayers * 2 + numFloors + ONE_SHADOW_LAYER as i32 + existsTagLayer(tagType) as i32
			{
				let mut layer = Vec::with_capacity(((xMax + 1) * (yMax + 1)) as _);
				layer.setLen(layer.capacity());
				cursor.read_u32_into::<LE>(&mut layer).unwrap();
				layers.push(layer);
			}
			let mut objects = Vec::with_capacity(cursor.read_i32::<LE>().unwrap() as _);
			for _ in 0..objects.capacity() {
				objects.push(Object {
					r#type: cursor.read_i32::<LE>().unwrap(),
					id: cursor.read_i32::<LE>().unwrap(),
					x: cursor.read_i32::<LE>().unwrap(),
					y: cursor.read_i32::<LE>().unwrap(),
					flags: cursor.read_i32::<LE>().unwrap(),
				});
			}
			let mut groups = Vec::with_capacity(if version >= 12 && existsTagLayer(tagType) {
				if version >= 18 {
					cursor.consumeZeros(size_of::<i32>());
				}
				cursor.read_i32::<LE>().unwrap() as _
			} else {
				0
			});
			for _ in 0..groups.capacity() {
				let x = cursor.read_i32::<LE>().unwrap();
				groups.push(Group {
					y: if cursor.remaining() == 0 {
						assert_eq!(x, 0);
						break;
					} else {
						cursor.read_i32::<LE>().unwrap()
					},
					x,
					width: cursor.read_i32::<LE>().unwrap(),
					height: cursor.read_i32::<LE>().unwrap(),
					unknown: if version < 13 { 0 } else { cursor.read_i32::<LE>().unwrap() },
				});
			}
			let mut paths = Vec::with_capacity(if version < 14 || cursor.remaining() == 0 {
				0
			} else {
				cursor.read_i32::<LE>().unwrap() as _
			});
			for _ in 0..paths.capacity() {
				let mut nodes = Vec::with_capacity(cursor.read_i32::<LE>().unwrap() as _);
				let [x, y] = array::from_fn(|_| cursor.read_i32::<LE>().unwrap());
				for _ in 0..nodes.capacity() {
					nodes.push(Node {
						x: cursor.read_i32::<LE>().unwrap(),
						y: cursor.read_i32::<LE>().unwrap(),
						action: if version < 15 { 1 } else { cursor.read_i32::<LE>().unwrap() },
					});
				}
				paths.push(Path { x, y, nodes });
			}

			trait Int??<ReturnedType> {
				fn int??(self) -> ReturnedType;
			}
			{
				type ReturnedType<T> = Option<Vec<T>>;
				impl<T> Int??<ReturnedType<T>> for Vec<T> {
					fn int??(self) -> ReturnedType<T> {
						if self.len() > 0 {
							Some(self)
						} else {
							None
						}
					}
				}
			}

			Ok(Self {
				version,
				xMax,
				yMax,
				actIndex,
				tagType,
				files,
				unknown,
				numWallLayers,
				numFloors,
				layers,
				objects: objects.int??(),
				groups: groups.int??(),
				paths: paths.int??(),
			})
		}
	}
}

pub const PAL_LEN: usize = 256 * 3;

pub mod dt1 {
	use {
		super::{log2, ReadExt, TileColumns},
		byteorder::{ReadBytesExt, LE},
		core::{
			cmp::{max, min},
			fmt, mem, ops,
		},
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

	impl Block {
		#[inline(always)]
		pub fn drawFn<T: DrawDestination>(&self) -> DrawFn<T> {
			if self.format == [1, 0] {
				DrawDestination::drawBlockIsometric
			} else {
				DrawDestination::drawBlockNormal
			}
		}
	}

	impl Tile {
		#[inline(always)]
		pub fn blockHeight(&self) -> usize {
			match self.orientation {
				FLOOR_ORIENTATION | ROOF_ORIENTATION => FLOOR_ROOF_BLOCKHEIGHT,
				_ => MAX_BLOCKHEIGHT,
			}
		}
	}

	impl Metadata {
		pub fn new(dt1: &[u8]) -> Result<Self, VersionMismatchError> {
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

			Ok(Self { fileHeader: FileHeader { version, tileHeadersPointer }, tiles })
		}
	}

	#[allow(non_camel_case_types)]
	type DrawFn<implDrawDestination> = fn(&mut implDrawDestination, x0: usize, y0: usize, data: &[u8]);

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

	pub const TILEWIDTH: usize = 160;
	pub const FLOOR_ROOF_TILEHEIGHT: usize = 79 + 1;
	pub const BLOCKWIDTH: usize = 32;
	pub const FLOOR_ROOF_BLOCKHEIGHT: usize = 15 + 1;
	pub const MAX_BLOCKHEIGHT: usize = 32;
	pub const FLOOR_ORIENTATION: i32 = 0;
	pub const ROOF_ORIENTATION: i32 = 15;

	impl super::Image {
		pub fn fromDT1(tiles: &[Tile], dt1: &[u8]) -> Self {
			let [mut minBlockHeight, mut maxBlockHeight] = [usize::MAX, 0];
			for tile in tiles {
				let blockHeight = tile.blockHeight();
				if blockHeight == 0 {
					continue;
				}
				minBlockHeight = min(minBlockHeight, blockHeight);
				maxBlockHeight = max(maxBlockHeight, blockHeight);
			}
			let height;
			let widthLog2 = {
				let chosenTileColumns = &{
					let choices = &mut Vec::<TileColumns>::new();
					choices.push(TileColumns {
						fullColumnHeight: maxBlockHeight,
						numOverflownColumns: 0,
						lastColumnHeight: 0,
					});
					for tile in tiles {
						let blockHeight = tile.blockHeight();
						for _ in &tile.blocks {
							choices.push(choices.last().unwrap().clone());
							let mut i = 0;
							while i < choices.len() {
								let result = choices[i].pushTile(blockHeight);
								if i == choices.len() - 2 {
									let lastIndex = choices.len() - 1;
									if result == 0 {
										choices.truncate(lastIndex);
									} else {
										assert_eq!(choices[lastIndex].numOverflownColumns, 0);
										choices[lastIndex].fullColumnHeight += minBlockHeight;
										choices.push(choices[lastIndex].clone());
									}
								}
								i += 1;
							}
						}
					}
					choices.sort_by(|a, b| {
						let dimensions = [a, b].map(|tileColumns| tileColumns.dimensions(BLOCKWIDTH));
						let pow2SquareSizes = dimensions.map(|[width, height]| max(width, height).next_power_of_two());
						const A: usize = 0;
						const B: usize = 1;
						const WIDTH: usize = 0;
						pow2SquareSizes[A]
							.cmp(&pow2SquareSizes[B])
							.then_with(|| dimensions[B][WIDTH].cmp(&dimensions[A][WIDTH]))
					});
					mem::take(&mut choices[0])
				};
				let width;
				[width, height] = chosenTileColumns.dimensions(BLOCKWIDTH);
				let pow2Width = width.next_power_of_two();
				eprintln!(
					"{}; {}",
					format_args!("[{width}, {height}] --> [{pow2Width}, {height}]"),
					format_args!(
						"lastColumnHeight = {}, minBlockHeight = {minBlockHeight}",
						chosenTileColumns.lastColumnHeight
					),
				);
				log2(pow2Width)
			};
			let mut image = Self { widthLog2, data: vec![0; height << widthLog2] };
			let (mut x, mut y) = (0, 0);
			for tile in tiles {
				for block in &tile.blocks {
					let nextY = {
						let blockHeight = tile.blockHeight();
						let nextY = y + blockHeight;
						if nextY > height {
							x += BLOCKWIDTH;
							y = 0;
							blockHeight
						} else {
							nextY
						}
					};
					block.drawFn()(
						&mut image,
						x,
						y,
						&dt1[(tile.blockHeadersPointer + block.fileOffset) as _..][..block.length as _],
					);
					y = nextY;
				}
			}
			image
		}
	}
}

use {
	dt1::BLOCKWIDTH,
	std::{
		fs::File,
		io::{self, Read},
		os,
	},
};

pub struct Image {
	pub widthLog2: usize,
	pub data: Vec<u8>,
}
impl Image {
	#[inline(always)]
	pub fn height(&self) -> usize {
		self.data.len() >> self.widthLog2
	}
	pub fn fromPNG(png: &mut png::Reader<impl Read>) -> Self {
		let widthLog2 = {
			let width = png.info().width as usize;
			assert!(width.is_power_of_two());
			log2(width)
		};
		let mut data = Vec::with_capacity(png.output_buffer_size());
		data.setLen(data.capacity());
		let len = png.next_frame(&mut data).unwrap().buffer_size();
		data.setLen(len);
		Self { widthLog2, data }
	}
	pub fn ??yBounds???BLOCKWIDTH???(&mut self, [x0, y0]: Vec2, height: usize) -> [i16; 2] {
		let [mut start??y, mut end??y] = [0, height];
		let [mut i, ??iNextLine] = [x0 + (y0 << self.widthLog2), 1 << self.widthLog2];
		const FULLY_TRANSPARENT_LINE: &[u8; BLOCKWIDTH] = &[FULLY_TRANSPARENT; BLOCKWIDTH];
		while start??y < end??y {
			if &self.data[i..][..BLOCKWIDTH] != FULLY_TRANSPARENT_LINE {
				break;
			}
			start??y += 1;
			i += ??iNextLine;
		}
		i = x0 + ((y0 + height - 1) << self.widthLog2);
		while start??y < end??y {
			if &self.data[i..][..BLOCKWIDTH] != FULLY_TRANSPARENT_LINE {
				break;
			}
			end??y -= 1;
			i -= ??iNextLine;
		}
		[start??y as _, end??y as _]
	}
	pub fn blitPixelsRectangle(
		&mut self,
		destPoint: Vec2,
		dimensions: Vec2,
		srcImage: &Self,
		srcPoint: Vec2,
	) {
		const WIDTH: usize = 0;
		const HEIGHT: usize = 1;
		let mut i = srcPoint[X] + (srcPoint[Y] << srcImage.widthLog2);
		let ??iNextLine = (1 << srcImage.widthLog2) - dimensions[WIDTH];
		let mut j = destPoint[X] + (destPoint[Y] << self.widthLog2);
		let ??jNextLine = (1 << self.widthLog2) - dimensions[WIDTH];
		let (mut ??x, mut ??y) = (0, 0);
		while ??y < dimensions[HEIGHT] {
			while ??x < dimensions[WIDTH] {
				match srcImage.data[i] {
					FULLY_TRANSPARENT => {}
					pixelValue => {
						assert_eq!(self.data[j], FULLY_TRANSPARENT);
						self.data[j] = pixelValue;
					}
				}
				??x += 1;
				i += 1;
				j += 1;
			}
			??x = 0;
			??y += 1;
			i += ??iNextLine;
			j += ??jNextLine;
		}
	}
}
pub type Vec2 = [usize; 2];
pub const X: usize = 0;
pub const Y: usize = 1;
pub trait Vec2Ext {
	fn add(self, rhs: Self) -> Self;
}
impl Vec2Ext for Vec2 {
	fn add(self, rhs: Self) -> Self {
		[self[0].wrapping_add(rhs[0]), self[1].wrapping_add(rhs[1])]
	}
}
pub const FULLY_TRANSPARENT: u8 = 0;
impl dt1::DrawDestination for Image {
	#[inline(always)]
	fn widthLog2(&self) -> usize {
		self.widthLog2
	}
	#[inline(always)]
	fn putpixel(&mut self, atIndex: usize, value: u8) {
		assert_ne!(value, FULLY_TRANSPARENT);
		self.data[atIndex] = value;
	}
}

#[derive(Debug, Default, Clone)]
pub struct TileColumns {
	pub fullColumnHeight: usize,
	pub numOverflownColumns: usize,
	pub lastColumnHeight: usize,
}
impl TileColumns {
	#[inline(always)]
	pub fn pushTile(&mut self, tileHeight: usize) -> usize {
		assert!(tileHeight <= self.fullColumnHeight);
		let (lastColumnHeight, fullColumnHeight) = (&mut self.lastColumnHeight, &mut self.fullColumnHeight);
		*lastColumnHeight += tileHeight;
		let dividend = *lastColumnHeight - 1;
		let (quotient, remainder) = (dividend / *fullColumnHeight, dividend % *fullColumnHeight);
		if quotient == 0 {
			return 0;
		}
		self.numOverflownColumns += quotient;
		*lastColumnHeight = tileHeight;
		tileHeight - remainder
	}
	#[inline(always)]
	pub const fn dimensions(&self, tileWidth: usize) -> [usize; 2] {
		[(self.numOverflownColumns + 1) * tileWidth, self.fullColumnHeight]
	}
}

pub struct TilesIterator<const TILEWIDTH: usize>(pub TileColumns);
impl<const TILEWIDTH: usize> TilesIterator<TILEWIDTH> {
	#[inline(always)]
	pub fn new(image: &Image) -> Self {
		Self(TileColumns {
			fullColumnHeight: image.data.len() >> image.widthLog2,
			numOverflownColumns: 0,
			lastColumnHeight: 0,
		})
	}
	#[inline(always)]
	pub fn next(&mut self, tileHeight: usize) -> [usize; 2] {
		let tileColumns = self.0.clone();
		if self.0.pushTile(tileHeight) != 0 {
			[self.0.numOverflownColumns * TILEWIDTH, 0]
		} else {
			[tileColumns.numOverflownColumns * TILEWIDTH, tileColumns.lastColumnHeight]
		}
	}
}

pub trait ReadExt {
	fn consumeZeros(&mut self, zerosCount: usize);
	fn read_u8_array<const N: usize>(&mut self) -> [u8; N];
	fn remaining(&self) -> usize;
}
impl<T: AsRef<[u8]>> ReadExt for io::Cursor<T> {
	fn consumeZeros(&mut self, zerosCount: usize) {
		let position = self.position() as usize;
		self.set_position((position + zerosCount) as _);
		let underlyingSlice = self.get_ref().as_ref();
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
		let underlyingSlice = self.get_ref().as_ref();
		<[u8; N]>::try_from(&underlyingSlice[position..self.position() as _]).unwrap()
	}
	fn remaining(&self) -> usize {
		let underlyingSlice = self.get_ref().as_ref();
		underlyingSlice.len() - self.position() as usize
	}
}

pub trait UsizeExt {
	fn nextMultipleOf(self, rhs: Self) -> Self;
}
impl UsizeExt for usize {
	#[inline(always)]
	fn nextMultipleOf(self, rhs: Self) -> Self {
		match self % rhs {
			0 => self,
			r => self + (rhs - r),
		}
	}
}

#[inline(always)]
pub const fn log2(of: usize) -> usize {
	(usize::BITS - 1 - of.leading_zeros()) as _
}

#[macro_export]
macro_rules! stringifyId {
	($id: ident) => {{
		_ = $id;
		stringify!($id)
	}};
}

pub trait VecExt {
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
