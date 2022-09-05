# tileset_d2sw
Either:
```sh
$ ln -s "/full/path/to/Diablo II Shareware v 1.04"/*.mpq/extracted/data/global
$ cargo run --release
```
, or:
```sh
$ cargo build --release
$ tileset_d2sw=$(readlink -f target/release/tileset_d2sw)
$ cd "/full/path/to/Diablo II Shareware v 1.04"/*.mpq/extracted/data
$ "$tileset_d2sw"
```
In any case, if everything has worked out fine, you'll have some new tileset\_d2sw\_*.{png,toml} files in your current directory.
