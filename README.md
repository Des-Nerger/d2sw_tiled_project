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
               | tee >(target/release/dubsplit "$d/$b".dt1.toml >/dev/null) \
               | target/release/3_-_dt1TOML-blockPNG_into_tilePNG --zealous-vertical-packing \
                   >"$d/$b".tile.png
           done

$ i=1; find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.dt1" -print \
    | cargo run --release --offline --bin dubcat \
    | cargo run --release --offline --bin dt1s_into_usedPALIndicesFrequency

$ i=1; cargo build --release --offline --bin 1_-_ds1_into_ds1TOML \
    && find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]${i} -iname "*.ds1" -print0 \
         | while read -d $'\0' f; do
             [[ $f =~ ([^/]+)/([^/]+)[.][A-Za-z0-9]+$ ]]
             d="/tmp/d2_act${i}/${BASH_REMATCH[1]}"
             mkdir -p "$d"
             b="${BASH_REMATCH[2]}"
             echo -n "${BASH_REMATCH[1]}/$b " 1>&2
             target/release/1_-_ds1_into_ds1TOML <"$f" >"$d/$b".ds1.toml
           done

$ p=(/tmp/d2_act1/CAVES/Cave.roguelikeTile.png); p=${p[@]%.roguelikeTile.png}; \
    cargo run --release --offline --bin 4_-_floorRoofTilePNG_into_rhombPackedTilePNG \
    <$p.roguelikeTile.png >$p.rhombPackedRoguelikeTile.png

$ cargo run --release --offline --bin 4_-_floorRoofTilePNG_into_noisySquareTilePNG \
    <'/tmp/d2_act1/Crypt/Floor.tile.png' >'/tmp/d2_act1/Crypt/Floor.noisySquareTile.png'

$ p=Floor.rhombPackedTile; cargo run --release --offline --bin dubcat <<< $p.waifu2x.png \
    | cat - $p.png \
    | cargo run --release --offline --bin waifu2xPNG-originalIndexedPNG_into_fixedWaifu2xPNG \
        >$p.fixedWaifu2x.png

$ p=(/tmp/d2_act1/?rypt/?loor.tile.png); p=${p[@]%.tile.png}; \
    cargo run --release --offline --bin dubcat <<< $p.dt1.toml \
      | cat - $p.tile.png \
      | cargo run --release --offline --bin 4_-_dt1TOML-tilePNG_into_roguelikeNoisySquareTilePNG \
          >$p.roguelikeNoisySquareTile.png

$ cargo build --release --offline --bin dubsplit --bin 2_-_pngPAL-dt1_into_dt1TOML-blockPNG \
                                  --bin 3_-_dt1TOML-blockPNG_into_tilePNG \
                                  --bin dubcat --bin 4_-_dt1TOML-tilePNG_into_roguelikeNoisySquareTilePNG \
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
               | tee >(target/release/dubsplit "$d/$b".dt1.toml >/dev/null) \
               | target/release/3_-_dt1TOML-blockPNG_into_tilePNG \
                   >/dev/shm/tile.png \
               && printf "%*s" ${#p} "" 1>&2 \
		           && target/release/dubcat <<< "$d/$b".dt1.toml | cat - /dev/shm/tile.png \
		                | target/release/4_-_dt1TOML-tilePNG_into_roguelikeNoisySquareTilePNG \
		                    >"$d/$b".roguelikeNoisySquareTile.png
           done; if [[ -f /dev/shm/tile.png ]]; then rm -v /dev/shm/tile.png; fi

$ cargo build --release --offline --bin dubsplit --bin 2_-_pngPAL-dt1_into_dt1TOML-blockPNG \
                                  --bin 3_-_dt1TOML-blockPNG_into_tilePNG \
                                  --bin dubcat --bin 4_-_dt1TOML-tilePNG_into_roguelikeTilePNG \
                                  --bin 4_-_floorRoofTilePNG_into_rhombPackedTilePNG \
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
               | tee >(target/release/dubsplit "$d/$b".dt1.toml >/dev/null) \
               | target/release/3_-_dt1TOML-blockPNG_into_tilePNG \
                   >/dev/shm/tile.png \
               && printf "%*s" ${#p} "" 1>&2 \
		           && target/release/dubcat <<< "$d/$b".dt1.toml | cat - /dev/shm/tile.png \
		                | target/release/4_-_dt1TOML-tilePNG_into_roguelikeTilePNG \
		                | target/release/4_-_floorRoofTilePNG_into_rhombPackedTilePNG \
		                    >"$d/$b".rhombPackedRoguelikeTile.png
           done; if [[ -f /dev/shm/tile.png ]]; then rm -v /dev/shm/tile.png; fi

$ cargo build --release --offline --bin dt1_into_dt1 \
    && find "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]1 -iname "*.dt1" -print0 \
         | while read -d $'\0' f; do
             [[ $f =~ ([^/]+)/([^/]+)[.][A-Za-z0-9]+$ ]]
             p="${BASH_REMATCH[1]}/${BASH_REMATCH[2]} "
             echo -n "$p" 1>&2
             target/release/dt1_into_dt1 <"$f" >/dev/shm/tmp.dt1 && cmp "$f" /dev/shm/tmp.dt1 && echo OK
           done; if [[ -f /dev/shm/tmp.dt1 ]]; then rm -v /dev/shm/tmp.dt1; fi

$ p=(/tmp/d2_act1/?rypt/?loor.tile.png); p=${p[@]%.tile.png}; \
    cargo run --release --offline --bin dubcat <<< $p.dt1.toml \
      | cat - $p.tile.png \
      | cargo run --release --offline --bin 4_-_dt1TOML-tilePNG_into_dt1 \
          >$p.dt1

$ cargo build --release --offline --bin 1_-_ds1_into_ds1TOML \
                                  --bin 2_-_ds1TOML_into_ds1SetFloorTOML --bin 2_-_ds1TOML_into_ds1 \
    && j=0 && ls "$PATH_D2_EXTRACTED"/data/global/tiles/[Aa][Cc][Tt]1/[Cc]rypt/*.[Dd][Ss]1 \
         | while read f; do
             [[ $f =~ ([^/]+)/([^/]+)[.][A-Za-z0-9]+$ ]]
             d="/tmp/d2_act1/${BASH_REMATCH[1]}"
             mkdir -p "$d"
             b="${BASH_REMATCH[2]}"
             p="${BASH_REMATCH[1]}/$b "
             echo -n "$p" 1>&2
             target/release/1_-_ds1_into_ds1TOML <"$f" \
               | target/release/2_-_ds1TOML_into_ds1SetFloorTOML 7 $j \
               | target/release/2_-_ds1TOML_into_ds1 >"$d/$b".ds1
             ((j++))
           done
```
