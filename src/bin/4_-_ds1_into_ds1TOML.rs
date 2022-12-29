#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	core::mem::size_of,
	d2sw_tiled_project::{ds1, stdoutRaw, ReadExt},
	serde::ser,
	std::io::{self, BufRead, Read, Write},
};

fn main() -> Result<(), ds1::VersionMismatchError> {
	let cursor = &mut io::Cursor::new(readToVec(io::stdin()).unwrap());
	let ds1RootStruct = &ds1::RootStruct::new(cursor)?;
	let remaining = cursor.remaining();
	eprintln!("v{} {}", ds1RootStruct.version, remaining);
	(if remaining == size_of::<i32>() { ReadExt::consumeZeros } else { io::Cursor::consume })(
		cursor, remaining,
	);
	stdoutRaw()
		.write_all(&toml_toStringPretty(ds1RootStruct).unwrap_or_else(|err| panic!("{err}")).into_bytes())
		.unwrap();

	fn readToVec(mut reader: impl Read) -> io::Result<Vec<u8>> {
		let mut vec = Vec::new();
		reader.read_to_end(&mut vec)?;
		Ok(vec)
	}

	fn toml_toStringPretty<T: ?Sized + ser::Serialize>(value: &T) -> Result<String, toml::ser::Error> {
		let mut string = String::with_capacity(128);
		value.serialize((&mut toml::ser::Serializer::pretty(&mut string)).pretty_array(false))?;
		Ok(string)
	}

	Ok(())
}
