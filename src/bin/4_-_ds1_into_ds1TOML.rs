#![warn(clippy::pedantic, elided_lifetimes_in_paths, explicit_outlives_requirements)]
#![allow(non_snake_case, confusable_idents, mixed_script_confusables, uncommon_codepoints)]

use {
	d2sw_tiled_project::{ds1, stdoutRaw},
	serde::ser,
	std::io::{self, Read, Write},
};

fn main() -> Result<(), ds1::VersionMismatchError> {
	stdoutRaw()
		.write_all(
			&toml_toStringPretty(&ds1::RootStruct::new(&readToVec(io::stdin()).unwrap())?)
				.unwrap_or_else(|err| panic!("{err}"))
				.into_bytes(),
		)
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
