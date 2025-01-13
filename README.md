# neuro-ftl

WIP

## Installation

### Linux

1. Build for `x86_64-unknown-linux-gnu`
2. Run game with `LD_PRELOAD=/path/to/libneuro_ftl.so`

### Windows

1. Build it for `i686-pc-windows-gnu` (`i686-pc-windows-msvc` works as
   well, the game uses MinGW rather than MSVC but I think? the C ABI
   should be the same, hopefully)
2. Rename `steam_api.dll` to `steam_api.orig.dll`
3. Move `neuro_ftl.dll` to the original `steam_api.dll`' location.

## Logs

- The `RUST_LOG_FILE` env var controls the log file name.
- The `RUST_LOG` env var controls the log level (i.e.
  `trace`/`debug`/`info`/`warn`/`error`)

## Testing

You can use the [Neuro Simulator](https://github.com/chayleaf/rust-neuro-sama-game-api/tree/master/neuro-simulator)

## Misc

Stuff is indexed by its name, if the name repeats twice then (2) is
added to the second one.
