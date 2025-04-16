/// 入力と出力に指定する色データタイプ
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorType {
    /// RGB888形式 `[r: u8, g: u8, b: u8]`
    Rgb888,

    /// RGB565形式 `u16`
    Rgb565,

    /// RGB8888形式 `[r; u8, g: u8, b: u8, a: u8]`
    Rgba8888,
}

impl ColorType {
    /// ピクセルあたりのバイト数
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            ColorType::Rgb888 => 3,
            ColorType::Rgb565 => 2,
            ColorType::Rgba8888 => 4,
        }
    }
}
