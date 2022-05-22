RustySamovar
===============================

[EN](README.md) | 中文

某款动漫游戏的自定义服务器

支持的游戏版本：1.4.5x - 2.7.5x（取决于提供的协议定义和使用的密钥）

**提示**: Github 存储库是位于主存储库的镜像 [Invisible Internet Bublik](http://bublik.i2p). 
如果 Github 镜像炸了，请使用 I2P 访问主站点。

# 构建

## 工具包准备

您需要安装任何 C/C++ 工具包:

- 在 Windows 上，MS VS Community 或 MinGW 将完成这项工作;
- 在Linux / Unix上，你只需要gcc或clang

您还需要安装 Rust

- 在windows上，请查看 [棺方文档](https://www.rust-lang.org/tools/install) （真心推荐使用Liunx！！！）
- 在Linux上，使用系统包管理器安装 `rustc` 和 `cargo`

## 准备仓库齐全

下载或克隆以下所有存储库 (`proto`, `mhycrypt`, `RustySamovar`, `kcp`, `lua_serde`) 并将其解压成文件夹，放在同一个目录中

## 检索协议定义

查看 `proto` 中的说明，关于如何获取所需文件集的项目。

## 检索 SSL 证书和流量加密密钥

要生成 SSL 证书，您需要安装 `openssl` 

- 在Linux / Unix上，使用 `misc/ssl_stuff/get_cert.sh`
- Windows上，别他妈告诉我你不会用Windows安装程序和openssh指令

然后获取证书，继续

## 编译

就跟你躺在床上打飞机一样简单，执行 `cargo build` 即可

# 运行

## 准备

要运行游戏，您需要一些游戏文件:

- [Lua脚本](https://github.com/14eyes/YSLua), 选取里面的 `DecompiledLua/Lua` 然后扔到 `(Server Root)/data/lua/` 里即可
- [ExcelBinOutput configs](https://github.com/Dimbreath/GenshinData), 选取里面的 `ExcelBinOutput` 然后扔到
  `(Server Root)/data/json/game/` 里
- [BinOutput configs](https://github.com/radioegor146/gi-bin-output), 选取里面的 `2.5.52/Data/_BinOutput` 然后扔到
  `(Server Root)/data/json/game/` 里

或者，您可以使用 Bublik（鬼知道为什么现在这东西打不开了） 提供的工具自行转储所有内容

## 将游戏的流量重定向到服务器

- Fiddler Classic，懂我意思吧
- hosts也可以

## 启动服务器

只需创建一个使用管理员身份运行的cmd窗口，然后执行 `cargo run` 即可，当然在Linux / Unix上，请阅读下文

- 在 Windows 上，应该会自动弹出 UAC 提示并要求您提升服务器的权限。 如果没有发生，请以管理员身份运行服务器的可执行文件。
- 在 *nix 上，您需要授予服务器特定的功能。 您可以通过运行 `sudo setcap 'cap_net_bind_service=+ep' ./target/debug/RustySamovar` 来完成。 **请不要以root身份运行服务器！**
