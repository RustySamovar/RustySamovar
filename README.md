RustySamovar
===============================
Custom server for YuanShen / Genshin Impact video game.

Supported game versions: 1.4.5x - 2.7.5x (depends on protocol definitions provided and keys used)

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

Clone / download and unzip all project repositories (`proto`, `mhycrypt`, `RustySamovar`, `kcp`, `lua_serde`) into the same directory.

## Retrieving protocol definitions

Look at the instructions in the `proto` project on how to get the required file set.

## Retrieving an SSL certificate and traffic encryption keys

To generate an SSL certificate, you'll need `openssl` tool installed.

- On *nix, use `misc/ssl_stuff/get_cert.sh` script
- On Windows, TODO

To get the traffic encryption key, TODO.

## Compiling

Just plain and simple `cargo build`.

# Running

## Preparation

To run the game, you'll need some of the game's files:

- [Lua scripts](https://github.com/14eyes/YSLua), grab them from `DecompiledLua/Lua` subdirectory and put into `data/lua/` subfolder of
  the server
- [ExcelBinOutput configs](https://github.com/Dimbreath/GenshinData), grab them from `ExcelBinOutput` subdirectory and put into
  `data/json/game/` subfolder of the server
- [BinOutput configs](https://github.com/radioegor146/gi-bin-output), grab them from `2.5.52/Data/_BinOutput` subdirectory and put into
  `data/json/game/` subfolder of the server

Alternatively you can dump everything by yourself using tools available at Bublik.

## Redirecting the game's traffic to the server

The simplest method is by modifying the `hosts` file. Copy the contents from the provided file into your system-wide one. 
Note that you'll need to comment those lines as soon as you'll want to play on the official servers or access official 
resources (like web events or daily login rewards).

## Starting the server

Just `cargo run` but with a caveat. Server listens on privileged ports (80, 443), so it needs permissions for that.

- On Windows, UAC prompt should automatically pop up and ask you to elevate server's priviledges. If it's not happening, run the server's
  executable as admin.
- On *nix, you'll need to grant the server the specific capability. You can do it by running `sudo setcap 'cap_net_bind_service=+ep' ./target/debug/RustySamovar`. **Please don't run the server as root!**
