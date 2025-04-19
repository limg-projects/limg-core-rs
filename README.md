# limg-core
Limg 画像を読み書きするための`no_std`コアライブラリです。

読み書きは`RGB888`、`RGB565`、`RGBA8888`に対応しています。

## Usage 
`Cargo.toml`に以下を入れてください。

```toml
[dependencies]
limg-core = { git = "https://github.com/limg-projects/limg-core-rs", tag = "v0.1.0" }
```