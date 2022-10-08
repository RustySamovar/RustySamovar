RustySamovar
===============================

EN | [中文](README_CN.md)

Custom server for YuanShen / Genshin Impact video game.

Supported game versions: 1.4.5x - 3.1.5x (depends on protocol definitions provided and keys used)

**Note**: Github repo is a mirror of the main repo located at [Invisible Internet Bublik](http://bublik.i2p). 
In case Github mirror dies, use I2P to access the main site.

# Building

## Toolkit preparation

You'll need any C/C++ toolkit installed:

- On Windows, MS VS Community or MinGW will do the job;
- On *nix, you just need GCC / Clang

Also you'll need to install Rust.

- On Windows, refer to [official instructions](https://www.rust-lang.org/tools/install)
- On Linux, use system package manager to install `rustc` and `cargo`

## Preparing the workplace

Clone repository with `git clone --recurse-submodules <repo_url>`. This is required to initialize all submodules.

## Retrieving protocol definitions

Look at the instructions in the `proto` project on how to get the required file set.

## Retrieving traffic encryption keys

Refer to `Sapozhok`'s README about traffic encryption keys. Note that `RustySamovar` doesn't need SSL keys, only RSA and regional ones.

## Compiling

Just plain and simple `cargo build`.

# Running

## Preparation

To run the server, you'll need some of the game's files:

- [Lua scripts](https://github.com/14eyes/YSLua), grab them from `DecompiledLua/Lua` subdirectory and put into `data/lua/` subfolder of
  the server
- [ExcelBinOutput configs](https://github.com/Dimbreath/GenshinData), grab them from `ExcelBinOutput` subdirectory and put into
  `data/json/game/` subfolder of the server
- [BinOutput configs](https://github.com/radioegor146/gi-bin-output), grab them from `2.5.52/Data/_BinOutput` subdirectory and put into
  `data/json/game/` subfolder of the server

Alternatively you can dump everything by yourself using tools available at Bublik.

## Starting the server

Just `cargo run` will do the trick.
