(Not ready for the public yet, but I keep it here anyway to hold myself accountable / motivated and as a cloud backup.)
# d2sw_tiled_project
```bash
$ export RUSTFLAGS="$RUSTFLAGS -C prefer-dynamic" # optional
$ export PATH_D2_EXTRACTED="/full/path/to/Diablo II Shareware v 1.04"/*.mpq/extracted

$ i=1; cargo run --release --offline --bin 1_-_pal_into_pngPAL \
    <"$PATH_D2_EXTRACTED"/data/global/palette/[Aa][Cc][Tt]${i}/pal.dat >/dev/shm/act${i}_pngPAL.dat

$ cargo build --release --offline --bin dubsplit \
              --bin 2_-_pngPAL-dt1_into_dt1TOML-blockPNG --bin 3_-_dt1TOML-blockPNG_into_tilePNG \
    && find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.dt1" -print0 \
         | while read -d $'\0' f; do
             [[ $f =~ ([^/]+)/([^/]+)[.][A-Za-z0-9]+$ ]]
             d="/tmp/d2_act${i}/${BASH_REMATCH[1]}"
             mkdir -p "$d"
             b="${BASH_REMATCH[2]}"
             p="${BASH_REMATCH[1]}/$b "
             echo -n "$p" 1>&2
             cat /dev/shm/act${i}_pngPAL.dat "$f" \
               | { target/release/2_-_pngPAL-dt1_into_dt1TOML-blockPNG && printf "%*s" ${#p} "" 1>&2 ; } \
               | tee >(target/release/dubsplit "$d/$b".dt1.toml >"$d/$b".block.png) \
               | target/release/3_-_dt1TOML-blockPNG_into_tilePNG >"$d/$b".tile.png
           done

$ i=1; find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.dt1" -print \
    | cargo run --release --offline --bin dubcat \
    | cargo run --release --offline --bin dt1s_into_usedPALIndicesFrequency

$ i=1; cargo build --release --offline --bin ds1_into_ds1TOML \
    && find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.ds1" -print0 \
         | while read -d $'\0' f; do
             [[ $f =~ ([^/]+)/([^/]+)[.][A-Za-z0-9]+$ ]]
             d="/tmp/d2_act${i}/${BASH_REMATCH[1]}"
             mkdir -p "$d"
             b="${BASH_REMATCH[2]}"
             echo -n "${BASH_REMATCH[1]}/$b " 1>&2
             target/release/ds1_into_ds1TOML <"$f" >"$d/$b".ds1.toml
           done

$ cargo run --release --offline --bin 4_-_floorRoofTilePNG_into_rhombPackedTilePNG <'/tmp/d2_act1/Crypt/Floor.tile.png' >'/tmp/d2_act1/Crypt/Floor.rhombPackedTile.png'
```
