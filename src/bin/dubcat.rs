#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case)]

use {
	core::{cmp::min, mem::transmute},
	d2sw_tiled_project::unbuffered_stdout,
	std::{
		fs::File,
		io::{self, BufRead, ErrorKind, Read, Write},
	},
};

fn main() {
	let (stdin, stdout) = (io::stdin(), &mut io::BufWriter::new(unbuffered_stdout()));
	let (mut stdin, filepathLine) = (stdin.lock(), &mut String::new());
	while {
		filepathLine.clear();
		stdin.read_line(filepathLine).unwrap() != 0
	} {
		let mut file = {
			let filepath = filepathLine.trim_end_matches(['\n', '\r']);
			File::open(filepath).unwrap_or_else(|err| panic!("{filepath:?}: {err}"))
		};
		let mut filesize = file.metadata().unwrap().len() as usize;
		writeln!(stdout, "{filesize}").unwrap();
		while filesize > 0 {
			let buffer = unsafe {
				struct BufWriter<W: Write> {
					_inner: W,
					buffer: Vec<u8>,
					_panicked: bool,
				}
				&mut (transmute::<&mut io::BufWriter<File>, &mut BufWriter<File>>(stdout).buffer)
			};
			if buffer.len() == buffer.capacity() {
				stdout.flush().unwrap();
			}
			let position = buffer.len();
			let numBytesToRead = min(buffer.capacity() - position, filesize);
			buffer.resizeUninitialized(position + numBytesToRead);
			let numBytesRead = file.readExactOrToEnd(&mut buffer[position..]).unwrap();
			if numBytesRead < numBytesToRead {
				buffer.truncate(position + numBytesRead);
				break;
			};
			filesize -= numBytesRead;

			trait VecExt {
				fn resizeUninitialized(&mut self, newLen: usize);
			}
			impl<T> VecExt for Vec<T> {
				fn resizeUninitialized(&mut self, newLen: usize) {
					if !cfg!(debug_assertions) {
						assert!(newLen <= self.capacity());
					}
					unsafe { self.set_len(newLen) }
				}
			}
			trait ReadExt {
				fn readExactOrToEnd(&mut self, buffer: &mut [u8]) -> io::Result<usize>;
			}
			impl<R: Read + ?Sized> ReadExt for R {
				fn readExactOrToEnd(&mut self, mut buffer: &mut [u8]) -> io::Result<usize> {
					let mut numBytesRead = 0;
					while !buffer.is_empty() {
						match self.read(buffer) {
							Ok(0) => break,
							Ok(n) => {
								buffer = &mut buffer[n..];
								numBytesRead += n;
							}
							Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
							Err(e) => return Err(e),
						}
					}
					Ok(numBytesRead)
				}
			}
		}
	}
}
