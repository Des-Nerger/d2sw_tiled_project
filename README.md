(Not ready for the public yet, but I keep it here anyway to hold myself accountable / motivated and as a cloud backup.)
# d2sw_tiled_project
```sh
$ export RUSTFLAGS="$RUSTFLAGS -C prefer-dynamic" # optional
$ find "/full/path/to/Diablo II Shareware v 1.04"/*.mpq/extracted/data/global/{palette,tiles}/[Aa]*1/ \
       -iname "*.d*t*" -print0 | \
  xargs -0 cargo run --release --
```
~~If everything has worked out fine, you'll have some newly generated `tileset_D2swAct1*.{png,toml}` files in your current directory.~~
