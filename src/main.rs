#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	byteorder::{ReadBytesExt, LE},
	const_format::concatcp,
	core::fmt,
	png::ColorType,
	std::{
		env,
		fs::File,
		io::{self, BufRead, BufWriter, IoSlice, Read, Write},
	},
};

fn main() {
	let mut args = env::args().skip(1);
	let palette = &mut Vec::<u8>::with_capacity(256 * 3);
	{
		let path: &str = &(args.next().unwrap());
		let mut file = File::open(path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
		file.read_to_end(palette).unwrap();
		assert_eq!(palette.capacity(), palette.len());
		for i in (0..palette.len()).step_by(3) {
			palette.swap(i + 0, i + 2);
		}
	}
	let palette: &_ = palette;
	let buffer = &mut Vec::<u8>::new();
	for path in args {
		let path = path.as_str();
		{
			let mut file = File::open(path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
			buffer.clear();
			file.read_to_end(buffer).unwrap();
		}
		let mut cursor = io::Cursor::new(buffer.as_slice());
		struct Pair<T, U>(T, U);
		{
			macro_rules! and {
				( $pair: ident . {0, 1} . $method: ident ( $($arg: ident)? ) ) => {
					$pair.0.$method($($arg)?).and($pair.1.$method($($arg)?))
				};
			}
			impl<T: Write, U: Write> Write for Pair<T, U> {
				fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
					and!(self.{0, 1}.write(buffer))
				}
				fn write_vectored(&mut self, buffers: &[IoSlice<'_>]) -> io::Result<usize> {
					and!(self.{0, 1}.write_vectored(buffers))
				}
				fn flush(&mut self) -> io::Result<()> {
					and!(self.{0, 1}.flush())
				}
				fn write_all(&mut self, buffer: &[u8]) -> io::Result<()> {
					and!(self.{0, 1}.write_all(buffer))
				}
				fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
					and!(self.{0, 1}.write_fmt(args))
				}
			}
		}
		const PATH_PREFIX: &str = "/dev/shm/tileset_D2swAct1TownFloor.";
		let toml =
			&mut Pair(BufWriter::new(File::create(concatcp!(PATH_PREFIX, "toml")).unwrap()), io::stdout());
		writeln!(toml, "[fileHeader]").unwrap();
		macro_rules! stringifyId {
			($id: ident) => {{
				_ = $id;
				stringify!($id)
			}};
		}
		let version = [cursor.read_i32::<LE>().unwrap(), cursor.read_i32::<LE>().unwrap()];
		writeln!(toml, "{} = {version:?}", stringifyId!(version)).unwrap();
		fn allZeros(byteSlice: &[u8]) -> bool {
			for &byte in byteSlice {
				if byte != 0 {
					return false;
				}
			}
			true
		}
		trait ConsumeZeros {
			fn consumeZeros(&mut self, zerosCount: usize);
		}
		impl ConsumeZeros for io::Cursor<&[u8]> {
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
		write!(
			toml,
			"{} = {numTiles}\n{} = {tileHeadersPointer}\n",
			stringifyId!(numTiles),
			stringifyId!(tileHeadersPointer)
		)
		.unwrap();
		const SUBTILE_SIZE: usize = 5;
		const NUM_SUBTILES: usize = SUBTILE_SIZE.pow(2);
		for _ in 0..numTiles {
			let direction = cursor.read_i32::<LE>().unwrap();
			let roofHeight = cursor.read_i16::<LE>().unwrap();
			let soundIndex = cursor.read_u8().unwrap();
			let isAnimated = match cursor.read_u8().unwrap() {
				0 => false,
				1 => true,
				byte => panic!("{}", byte),
			};
			let height = cursor.read_i32::<LE>().unwrap();
			let width = cursor.read_i32::<LE>().unwrap();
			writeln!(
				toml,
				"\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
				format_args!("[[tileHeader]]"),
				format_args!("{} = {direction}", stringifyId!(direction)),
				format_args!("{} = {roofHeight}", stringifyId!(roofHeight)),
				format_args!("{} = {soundIndex}", stringifyId!(soundIndex)),
				format_args!("{} = {isAnimated}", stringifyId!(isAnimated)),
				format_args!("{} = {height}", stringifyId!(height)),
				format_args!("{} = {width}", stringifyId!(width)),
			)
			.unwrap();
			cursor.consumeZeros(4);
			let orientation = cursor.read_i32::<LE>().unwrap();
			let mainIndex = cursor.read_i32::<LE>().unwrap();
			let subIndex = cursor.read_i32::<LE>().unwrap();
			let rarityFrameIndex = cursor.read_i32::<LE>().unwrap();
			let unknown = [cursor.read_i16::<LE>().unwrap(), cursor.read_i16::<LE>().unwrap()];
			let mut subtileFlags = [0xFF_u8; NUM_SUBTILES];
			cursor.read_exact(&mut subtileFlags).unwrap();
			cursor.consumeZeros(7);
			let blockHeadersPointer = cursor.read_i32::<LE>().unwrap();
			let blockDataLength = cursor.read_i32::<LE>().unwrap();
			let numBlocks = cursor.read_i32::<LE>().unwrap();
			assert_eq!(numBlocks as usize, NUM_SUBTILES);
			writeln!(
				toml,
				"id = {{{}, {}, {}}}\n{}\n{}\n{}\n{}\n{}\n{}",
				format_args!("{} = {orientation}", stringifyId!(orientation)),
				format_args!("{} = {mainIndex}", stringifyId!(mainIndex)),
				format_args!("{} = {subIndex}", stringifyId!(subIndex)),
				format_args!("{} = {rarityFrameIndex}", stringifyId!(rarityFrameIndex)),
				format_args!("{} = {unknown:?}", stringifyId!(unknown)),
				format_args!("{} = {subtileFlags:#04X?}", stringifyId!(subtileFlags)),
				format_args!("{} = {blockHeadersPointer}", stringifyId!(blockHeadersPointer)),
				format_args!("{} = {blockDataLength}", stringifyId!(blockDataLength)),
				format_args!("{} = {numBlocks}", stringifyId!(numBlocks)),
			)
			.unwrap();
			cursor.consumeZeros(12); // they're not all zeros all of the time; FIXME later
		}
		let mut imageData = [98_u8; 256 * 256];
		for i in 0..numTiles {
			let blockHeadersPointer = cursor.position() as usize;
			writeln!(toml, "\n[[current]]\n{} = {blockHeadersPointer}", stringifyId!(blockHeadersPointer))
				.unwrap();
			if i == 114
			// 114
			{
				break;
			}
			for j in 0..NUM_SUBTILES {
				let x = cursor.read_i16::<LE>().unwrap();
				let y = cursor.read_i16::<LE>().unwrap();
				cursor.consumeZeros(2);
				let gridX = cursor.read_u8().unwrap();
				let gridY = cursor.read_u8().unwrap();
				let format = cursor.read_i16::<LE>().unwrap();
				let length = cursor.read_i32::<LE>().unwrap() as usize;
				cursor.consumeZeros(2);
				let filePointer = blockHeadersPointer + cursor.read_i32::<LE>().unwrap() as usize;
				writeln!(
					toml,
					"\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
					format_args!("[[blockHeader]]"),
					format_args!("{} = {x}", stringifyId!(x)),
					format_args!("{} = {y}", stringifyId!(y)),
					format_args!("{} = {gridX}", stringifyId!(gridX)),
					format_args!("{} = {gridY}", stringifyId!(gridY)),
					format_args!("{} = {format}", stringifyId!(format)),
					format_args!("{} = {length}", stringifyId!(length)),
					format_args!("{} = {filePointer}", stringifyId!(filePointer)),
				)
				.unwrap();
				if i != 113 {
					continue;
				}
				(if format == 1 { drawBlockIsometric } else { drawBlockNormal })(
					&mut imageData,
					32 * (j % 5),
					16 * (j / 5),
					&buffer[filePointer..filePointer + length],
				);
			}
			writeln!(toml, "\n[[current]]\ncursor.position = {}", cursor.position()).unwrap();
			cursor.consume(256 * NUM_SUBTILES);
		}
		let mut encoder =
			png::Encoder::new(BufWriter::new(File::create(concatcp!(PATH_PREFIX, "png")).unwrap()), 256, 256);
		encoder.set_color(ColorType::Indexed);
		encoder.set_palette(palette);
		encoder.write_header().unwrap().write_image_data(&imageData).unwrap();
	}
}

/*
	3D-isometric Block :

	1st line : draw a line of 4 pixels
	2nd line : draw a line of 8 pixels
	3rd line : draw a line of 12 pixels
	and so on...
*/
fn drawBlockIsometric(dst: &mut [u8], x0: usize, y0: usize, data: &[u8]) {
	let mut length = data.len();

	// 3d-isometric subtile is 256 bytes, no more, no less
	assert_eq!(length, 256);

	// draw
	let (mut i, mut y) = (0, 0);
	while length > 0 {
		static XJUMP: [u8; 15] = [14, 12, 10, 8, 6, 4, 2, 0, 2, 4, 6, 8, 10, 12, 14];
		static NBPIX: [u8; 15] = [4, 8, 12, 16, 20, 24, 28, 32, 28, 24, 20, 16, 12, 8, 4];
		let (mut j, mut n) = ((y0 + y) * 256 + x0 + XJUMP[y] as usize, NBPIX[y] as usize);
		length -= n;
		while n != 0 {
			dst[j] = data[i];
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
fn drawBlockNormal(dst: &mut [u8], x0: usize, y0: usize, data: &[u8]) {
	let mut length = data.len();

	// draw
	let (mut i, mut y, j0) = (0, 0, |y| (y0 + y) * 256 + x0);
	let mut j = j0(y);
	while length > 0 {
		let (xjump, mut xsolid) = (data[i + 0] as usize, data[i + 1] as usize);
		i += 2;
		length -= 2;
		if xjump != 0 || xsolid != 0 {
			j += xjump;
			length -= xsolid;
			while xsolid != 0 {
				dst[j] = data[i];
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
