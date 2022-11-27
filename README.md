(Not ready for the public yet, but I keep it here anyway to hold myself accountable / motivated and as a cloud backup.)
# d2sw_tiled_project
```bash
$ export RUSTFLAGS="$RUSTFLAGS -C prefer-dynamic" # optional
$ export PATH_D2_EXTRACTED="/full/path/to/Diablo II Shareware v 1.04"/*.mpq/extracted

$ i=1; cargo run --release --offline --bin 1_-_pal_into_swappedPAL \
    <"$PATH_D2_EXTRACTED"/data/global/palette/[Aa][Cc][Tt]${i}/pal.dat >/dev/shm/act${i}_swappedPAL.dat

$ cargo build --release --offline --bin dubsplit \
              --bin 2_-_swappedPAL-dt1_into_toml-blockPNG --bin 3_-_toml-blockPNG_into_tilePNG \
    && find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.dt1" -print0 \
         | while read -d $'\0' f; do
             [[ $f =~ ([^/]+)/([^/]+)[.][A-Za-z0-9]+$ ]]
             d="/tmp/d2_act${i}/${BASH_REMATCH[1]}"
             mkdir -p "$d"
             b="${BASH_REMATCH[2]}"
             echo -n "${BASH_REMATCH[1]}/$b "
             cat /dev/shm/act${i}_swappedPAL.dat "$f" \
               | target/release/2_-_swappedPAL-dt1_into_toml-blockPNG \
               | tee >(target/release/dubsplit "$d/$b".toml >"$d/$b".block.png) \
               | target/release/3_-_toml-blockPNG_into_tilePNG >"$d/$b".tile.png
           done

$ find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.dt1" -print \
    | cargo run --release --offline --bin dubcat \
    | cargo run --release --offline --bin dt1s_into_usedPALIndicesFrequency
```
