#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

pub mod ds1 {
	use {
		super::{ReadExt, VecExt},
		byteorder::{ReadBytesExt, WriteBytesExt, LE},
		core::{
			array, fmt,
			mem::{size_of, size_of_val},
			slice,
		},
		memchr::memchr,
		serde::{Deserialize, Serialize},
		std::io::{self, BufRead, Write},
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

	pub const MAIN_INDEX_OFFSET: u32 =
		(0b1111_i32).trailing_ones() + (SUB_INDEX_MASK | LAYER_DRAWING_PRIORITY_MASK).trailing_ones();
	pub const MAIN_INDEX_MASK: u32 = MAIN_INDEX_MAX << MAIN_INDEX_OFFSET;
	pub const MAIN_INDEX_MAX: u32 = 0b0011_1111;
	pub const SUB_INDEX_OFFSET: u32 = LAYER_DRAWING_PRIORITY_MASK.trailing_ones();
	pub const SUB_INDEX_MASK: u32 = SUB_INDEX_MAX << SUB_INDEX_OFFSET;
	pub const SUB_INDEX_MAX: u32 = 0b1111_1111;
	pub const LAYER_DRAWING_PRIORITY_MASK: u32 = PROP1_MASK;
	pub const ORIENTATION_MASK: u32 = PROP1_MASK;
	const PROP1_MASK: u32 = 0b1111_1111;
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
		pub fn writeTo(&self, to: &mut impl Write) {
			let &RootStruct {
				version,
				xMax,
				yMax,
				actIndex,
				tagType,
				ref files,
				ref unknown,
				numWallLayers,
				numFloors,
				ref layers,
				ref objects,
				ref groups,
				ref paths,
			} = self;
			to.write_i32::<LE>(version).unwrap();
			[xMax, yMax].iter().for_each(|&coordMax| to.write_i32::<LE>(coordMax).unwrap());
			if version >= 8 {
				to.write_i32::<LE>(actIndex).unwrap();
				if version >= 10 {
					to.write_i32::<LE>(tagType).unwrap();
				}
			}
			to.write_i32::<LE>(files.len() as _).unwrap();
			for file in files {
				to.write_all(file.as_bytes()).unwrap();
				to.write_u8(b'\0').unwrap();
			}
			if matches!(version, 9..=13) {
				to.write_all(&unknown.unwrap()).unwrap();
			}
			to.write_i32::<LE>(numWallLayers).unwrap();
			if version >= 16 {
				to.write_i32::<LE>(numFloors).unwrap();
			}
			for layer in layers {
				if cfg!(target_endian = "little") {
					to.write_all(unsafe {
						slice::from_raw_parts(layer.as_ptr() as _, size_of_val(layer.as_slice()))
					})
					.unwrap();
				} else {
					for &cell in layer {
						_ = to.write_u32::<LE>(cell);
					}
				}
			}
			let (objects, groups, paths) = (objects.toVec(), groups.toVec(), paths.toVec());
			to.write_i32::<LE>(objects.len() as _).unwrap();
			for &Object { r#type, id, x, y, flags } in objects {
				to.write_i32::<LE>(r#type).unwrap();
				to.write_i32::<LE>(id).unwrap();
				[x, y].iter().for_each(|&coord| to.write_i32::<LE>(coord).unwrap());
				to.write_i32::<LE>(flags).unwrap();
			}
			if version >= 12 {
				if existsTagLayer(tagType) {
					if version >= 18 {
						to.write_i32::<LE>(0).unwrap();
					}
					to.write_i32::<LE>(groups.len() as _).unwrap();
					for &Group { x, y, width, height, unknown } in groups {
						[x, y].iter().for_each(|&coord| to.write_i32::<LE>(coord).unwrap());
						[width, height].iter().for_each(|&dimension| to.write_i32::<LE>(dimension).unwrap());
						if version >= 13 {
							to.write_i32::<LE>(unknown).unwrap();
						}
					}
				}
				if version >= 14 {
					to.write_i32::<LE>(paths.len() as _).unwrap();
					for &Path { x, y, ref nodes } in paths {
						to.write_i32::<LE>(nodes.len() as _).unwrap();
						[x, y].iter().for_each(|&coord| to.write_i32::<LE>(coord).unwrap());
						for &Node { x, y, action } in nodes {
							[x, y].iter().for_each(|&coord| to.write_i32::<LE>(coord).unwrap());
							if version >= 15 {
								to.write_i32::<LE>(action).unwrap()
							}
						}
					}
				}
			}

			trait OptionVecExt<T: 'static> {
				fn toVec(&self) -> &Vec<T>;
				const EMPTY_VEC: &'static Vec<T> = &Vec::new();
			}
			impl<T: 'static> OptionVecExt<T> for Option<Vec<T>> {
				fn toVec(&self) -> &Vec<T> {
					self.as_ref().unwrap_or(Self::EMPTY_VEC)
				}
			}
		}

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
				let mut layer = Vec::withLen(((xMax + 1) * (yMax + 1)) as _);
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

			trait Intо<ReturnedType> {
				fn intо(self) -> ReturnedType;
			}
			{
				type ReturnedType<T> = Option<Vec<T>>;
				impl<T> Intо<ReturnedType<T>> for Vec<T> {
					fn intо(self) -> ReturnedType<T> {
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
				objects: objects.intо(),
				groups: groups.intо(),
				paths: paths.intо(),
			})
		}
	}
}

pub const PAL_LEN: usize = 256 * 3;

pub mod dt1 {
	use {
		super::{
			CopyExt, Image, ReadExt, TileColumns, TilesIterator, UsizeExt, Vec2, Vec2Ext, WriteExt,
			FULLY_TRANSPARENT, X, Y,
		},
		byteorder::{ReadBytesExt, WriteBytesExt, LE},
		core::{
			cmp::{max, min},
			fmt, iter, mem, ops,
		},
		serde::{Deserialize, Serialize},
		std::io::{self, BufRead, Write},
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

	pub const NUM_SUBTILES_PER_LINE: usize = 5;
	const NUM_SUBTILES: usize = NUM_SUBTILES_PER_LINE.pow(2);
	const FILEHEADER_SIZE: i32 = 276;
	const TILEHEADER_SIZE: i32 = 96;
	const BLOCKHEADER_SIZE: i32 = 20;

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
					#[allow(non_camel_case_types)]
					trait Copy_AddAssign_Ext {
						fn alsoAddTo(self, to: &mut Self) -> Self;
					}
					impl<T: Copy + CopyExt + ops::AddAssign> Copy_AddAssign_Ext for T {
						#[inline(always)]
						fn alsoAddTo(self, to: &mut Self) -> Self {
							self.also(|&Δ| *to += Δ)
						}
					}
				}
				assert_eq!(blocks.len(), blocks.capacity());
				cursor.consume(totalLength as _);
			}
			assert_eq!(cursor.position(), dt1.len() as _);
			Ok(Self { fileHeader: FileHeader { version, tileHeadersPointer }, tiles })
		}

		pub fn writeWithBlockDataFromTileImage(&self, tileImage: &Image, to: &mut impl Write) {
			let Self { fileHeader, tiles } = self;
			fileHeader.version.iter().for_each(|&versionElem| to.write_i32::<LE>(versionElem).unwrap());
			to.writeZeros(260);
			to.write_i32::<LE>(tiles.len() as _).unwrap();
			to.write_i32::<LE>(fileHeader.tileHeadersPointer).unwrap();
			{
				let mut blockHeadersPointer = FILEHEADER_SIZE + tiles.len() as i32 * TILEHEADER_SIZE;
				for &Tile {
					direction,
					roofHeight,
					ref materialFlags,
					height,
					width,
					orientation,
					mainIndex,
					subIndex,
					rarityOrFrameIndex,
					ref unknown,
					ref subtileFlags,
					blockHeadersPointer: _,
					blockDataLength,
					ref usuallyZeros,
					ref blocks,
				} in tiles
				{
					to.write_i32::<LE>(direction).unwrap();
					to.write_i16::<LE>(roofHeight).unwrap();
					to.write_all(materialFlags).unwrap();
					to.write_i32::<LE>(height).unwrap();
					to.write_i32::<LE>(width).unwrap();
					to.writeZeros(4);
					to.write_i32::<LE>(orientation).unwrap();
					to.write_i32::<LE>(mainIndex).unwrap();
					to.write_i32::<LE>(subIndex).unwrap();
					to.write_i32::<LE>(rarityOrFrameIndex).unwrap();
					to.write_all(unknown).unwrap();
					to.write_all(subtileFlags).unwrap();
					to.writeZeros(7);
					to.write_i32::<LE>(blockHeadersPointer).unwrap();
					to.write_i32::<LE>(blockDataLength).unwrap();
					to.write_i32::<LE>(blocks.len() as _).unwrap();
					to.writeZeros(4);
					to.write_all(usuallyZeros).unwrap();
					to.writeZeros(4);
					blockHeadersPointer += blockDataLength;
				}
			}
			let points = &mut TilesIterator::<{ TILEWIDTH }>::new(tileImage);
			for tile in tiles {
				let ([mut startY, mut endY, blockHeight], mut fileOffset, blocks) = (
					[i16::MAX, i16::MIN, tile.blockHeight() as _],
					tile.blocks.len() as i32 * BLOCKHEADER_SIZE,
					&tile.blocks,
				);
				for &Block { x, y, gridX, gridY, ref format, length, fileOffset: _ } in blocks {
					to.write_i16::<LE>(x).unwrap();
					to.write_i16::<LE>(y).unwrap();
					to.writeZeros(2);
					to.write_u8(gridX).unwrap();
					to.write_u8(gridY).unwrap();
					to.write_all(format).unwrap();
					to.write_i32::<LE>(length).unwrap();
					to.writeZeros(2);
					to.write_i32::<LE>(fileOffset).unwrap();
					startY = min(startY, y);
					endY = max(endY, y + blockHeight);
					fileOffset += length;
				}
				let point = points
					.next(((endY - startY) as usize).nextMultipleOf(FLOOR_ROOF_BLOCKHEIGHT))
					.add([0, 0_usize.wrapping_sub(startY as _)]);
				for &Block { x, y, .. } in blocks {
					to.writeBlockIsometric(point.add([x as _, y as _]), tileImage);
				}
			}
			trait WriteBlockExt {
				fn writeBlockIsometric(&mut self, point: Vec2, tileImage: &Image);
			}
			impl<T: Write> WriteBlockExt for T {
				fn writeBlockIsometric(&mut self, point: Vec2, tileImage: &Image) {
					let mut i = point[Y] * tileImage.width + point[X];
					for (xjump, nbpix) in iter::zip(XJUMP, NBPIX) {
						_ = self.write_all(&tileImage.data[i + xjump..][..nbpix]);
						i += tileImage.width;
					}
				}
			}
		}
	}

	const XJUMP: [usize; 15] = [14, 12, 10, 8, 6, 4, 2, 0, 2, 4, 6, 8, 10, 12, 14];
	const NBPIX: [usize; 15] = [4, 8, 12, 16, 20, 24, 28, 32, 28, 24, 20, 16, 12, 8, 4];

	type DrawFn<ImplDrawDestination> = fn(&mut ImplDrawDestination, x0: usize, y0: usize, data: &[u8]);

	pub trait DrawDestination {
		fn width(&self) -> usize;
		fn putpixel(&mut self, atIndex: usize, value: u8);

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
			let [mut Δy, width] = [0, self.width()];
			let [mut i, mut yMulWidthAddX0] = [0, y0 * width + x0];
			while length > 0 {
				let [mut j, mut n] = [yMulWidthAddX0 + XJUMP[Δy], NBPIX[Δy]];
				length -= n;
				while n != 0 {
					self.putpixel(j, data[i]);
					i += 1;
					j += 1;
					n -= 1;
				}
				Δy += 1;
				yMulWidthAddX0 += width;
			}
		}

		/*
			RLE Block :

			1st byte is pixels to "jump", 2nd is number of "solid" pixels, followed by the pixel color indexes.
			when 1st and 2nd bytes are 0 and 0, next line.
		*/
		fn drawBlockNormal(&mut self, x0: usize, y0: usize, data: &[u8]) {
			let [mut length, width] = [data.len(), self.width()];

			// draw
			let (mut i, [mut j, mut yMulWidthAddX0]) = (0, [y0 * width + x0; 2]);
			while length > 0 {
				let [xjump, mut xsolid] = [data[i + 0] as usize, data[i + 1] as usize];
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
					yMulWidthAddX0 += width;
					j = yMulWidthAddX0;
				}
			}
		}
	}

	pub const TILEWIDTH: usize = 160;
	pub const FLOOR_ROOF_TILEHEIGHT: usize = 79 + 1;
	pub const BLOCKWIDTH: usize = 32;
	pub const FLOOR_ROOF_BLOCKHEIGHT: usize = 15 + 1;
	pub const MAX_BLOCKHEIGHT: usize = 32;

	pub const SQUARE_TILE_SIZE: usize = TILEWIDTH / 2;
	pub const SQUARE_SUBTILE_SIZE: usize = BLOCKWIDTH / 2;

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
			let (width, height);
			{
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
				[width, height] = chosenTileColumns.dimensions(BLOCKWIDTH);
				eprintln!(
					"[{width}, {height}]; lastColumnHeight = {}, minBlockHeight = {minBlockHeight}",
					chosenTileColumns.lastColumnHeight,
				);
			}
			let mut image = Self::fromWidthHeight(width, height);
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

		pub fn drawNoisySquareTile(&mut self, mut destPoint: Vec2, srcImage: &Self, mut srcPoint: Vec2) {
			destPoint[Y] += 1;
			srcPoint[X] += SQUARE_TILE_SIZE - 1;
			let [mut iY, mut jY] = [srcPoint[Y] * srcImage.width, destPoint[Y] * self.width];
			for Δx in 0..SQUARE_TILE_SIZE {
				let [mut i, mut j] = [srcPoint[X] + iY, destPoint[X] + jY];
				for Δy in 0..SQUARE_TILE_SIZE {
					match srcImage.data[i] {
						FULLY_TRANSPARENT => {}
						pixelValue => {
							assert_eq!(self.data[j], FULLY_TRANSPARENT);
							self.data[j] = pixelValue;
						}
					}
					i += usize::MAX + if Δy % 2 == 0 { 0 } else { srcImage.width };
					j += self.width;
				}
				destPoint.addAssign([
					1,
					(if Δx % 2 == 0 {
						srcPoint[X] += 2;
						usize::MAX
					} else {
						srcPoint[Y] += 1;
						iY += srcImage.width;
						1
					})
					.also(|&Δy| jY += self.width.mulSignumOf(Δy)),
				]);
			}
		}
	}
}

use {
	dt1::BLOCKWIDTH,
	serde::ser,
	std::{
		fs::File,
		io::{self, Read, Write},
		os,
	},
};

pub struct Image {
	pub width: usize,
	pub height: usize,
	pub data: Vec<u8>,
}
impl Image {
	#[inline(always)]
	pub fn fromWidthData(width: usize, data: Vec<u8>) -> Self {
		Self {
			width,
			height: {
				let len = data.len();
				assert_eq!(len % width, 0);
				len / width
			},
			data,
		}
	}
	#[inline(always)]
	pub fn fromWidthHeight(width: usize, height: usize) -> Self {
		Self { width, height, data: vec![FULLY_TRANSPARENT; width * height] }
	}
	pub fn fromPNG(png: &mut png::Reader<impl Read>) -> Self {
		let mut data = Vec::withLen(png.output_buffer_size());
		let len = png.next_frame(&mut data).unwrap().buffer_size();
		data.setLen(len);
		Self::fromWidthData(png.info().width as _, data)
	}
	pub fn ΔyBoundsᐸBLOCKWIDTHᐳ(&mut self, [x0, y0]: Vec2, height: usize) -> [i16; 2] {
		let [mut startΔy, mut endΔy, width] = [0, height, self.width];
		let mut i = x0 + y0 * width;
		const FULLY_TRANSPARENT_LINE: &[u8; BLOCKWIDTH] = &[FULLY_TRANSPARENT; BLOCKWIDTH];
		while startΔy < endΔy {
			if &self.data[i..][..BLOCKWIDTH] != FULLY_TRANSPARENT_LINE {
				break;
			}
			startΔy += 1;
			i += width;
		}
		i = x0 + (y0 + height - 1) * width;
		while startΔy < endΔy {
			if &self.data[i..][..BLOCKWIDTH] != FULLY_TRANSPARENT_LINE {
				break;
			}
			endΔy -= 1;
			i -= width;
		}
		[startΔy as _, endΔy as _]
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
		let mut i = srcPoint[X] + srcPoint[Y] * srcImage.width;
		let ΔiNextLine = srcImage.width - dimensions[WIDTH];
		let mut j = destPoint[X] + destPoint[Y] * self.width;
		let ΔjNextLine = self.width - dimensions[WIDTH];
		let mut Δy = 0;
		while Δy < dimensions[HEIGHT] {
			let mut Δx = 0;
			while Δx < dimensions[WIDTH] {
				match srcImage.data[i] {
					FULLY_TRANSPARENT => {}
					pixelValue => {
						// assert_eq!(self.data[j], FULLY_TRANSPARENT);
						self.data[j] = pixelValue;
					}
				}
				Δx += 1;
				i += 1;
				j += 1;
			}
			Δy += 1;
			i += ΔiNextLine;
			j += ΔjNextLine;
		}
	}
}
pub type Vec2 = [usize; 2];
pub const X: usize = 0;
pub const Y: usize = 1;
pub trait Vec2Ext {
	fn add(self, rhs: Self) -> Self;
	fn addAssign(&mut self, rhs: Self);
}
impl Vec2Ext for Vec2 {
	#[inline(always)]
	fn add(self, rhs: Self) -> Self {
		[self[0].wrapping_add(rhs[0]), self[1].wrapping_add(rhs[1])]
	}

	#[inline(always)]
	fn addAssign(&mut self, rhs: Self) {
		*self = self.add(rhs);
	}
}
pub const FULLY_TRANSPARENT: u8 = 0;
impl dt1::DrawDestination for Image {
	#[inline(always)]
	fn width(&self) -> usize {
		self.width
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
		Self(TileColumns { fullColumnHeight: image.height, numOverflownColumns: 0, lastColumnHeight: 0 })
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
	#[inline(always)]
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
	#[inline(always)]
	fn read_u8_array<const N: usize>(&mut self) -> [u8; N] {
		let position = self.position() as usize;
		self.set_position((position + N) as _);
		let underlyingSlice = self.get_ref().as_ref();
		<[u8; N]>::try_from(&underlyingSlice[position..self.position() as _]).unwrap()
	}
	#[inline(always)]
	fn remaining(&self) -> usize {
		let underlyingSlice = self.get_ref().as_ref();
		underlyingSlice.len() - self.position() as usize
	}
}

trait WriteExt {
	fn writeZeros(&mut self, zerosCount: u64);
}
impl<T: Write> WriteExt for T {
	#[inline(always)]
	fn writeZeros(&mut self, zerosCount: u64) {
		io::copy(&mut io::repeat(0).take(zerosCount), self).unwrap();
	}
}

pub trait UsizeExt {
	fn nextMultipleOf(self, rhs: Self) -> Self;
	fn mulSignumOf(self, rhs: Self) -> Self;
}
impl UsizeExt for usize {
	#[inline(always)]
	fn nextMultipleOf(self, rhs: Self) -> Self {
		match self % rhs {
			0 => self,
			r => self + (rhs - r),
		}
	}
	#[inline(always)]
	fn mulSignumOf(self, rhs: Self) -> Self {
		let signBits = (rhs as isize >> (usize::BITS - 1)) as usize;
		(self ^ signBits) - signBits
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
	fn withLen(len: usize) -> Self;
	fn setLen(&mut self, newLen: usize);
}
impl<T> VecExt for Vec<T> {
	#[inline(always)]
	fn withLen(len: usize) -> Self {
		let mut sеlf = Self::with_capacity(len);
		sеlf.setLen(len);
		sеlf
	}
	#[inline(always)]
	fn setLen(&mut self, newLen: usize) {
		if !cfg!(debug_assertions) {
			assert!(newLen <= self.capacity());
		}
		unsafe { self.set_len(newLen) };
	}
}

pub trait CopyExt {
	fn also(self, f: impl FnOnce(&Self)) -> Self;
}
impl<T: Copy> CopyExt for T {
	#[inline(always)]
	fn also(self, f: impl FnOnce(&Self)) -> Self {
		f(&self);
		self
	}
}

#[inline(always)]
pub fn io_readToString(mut reader: impl Read) -> io::Result<String> {
	let mut string = String::new();
	reader.read_to_string(&mut string)?;
	Ok(string)
}

#[inline(always)]
pub fn toml_toStringPretty<T: ?Sized + ser::Serialize>(value: &T) -> Result<String, toml::ser::Error> {
	let mut string = String::with_capacity(128);
	value.serialize((&mut toml::ser::Serializer::pretty(&mut string)).pretty_array(false))?;
	Ok(string)
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
