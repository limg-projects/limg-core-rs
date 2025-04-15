#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorType {
    Rgb888,
    Rgb565,
    Rgba8888,
}

impl ColorType {
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            ColorType::Rgb888 => 3,
            ColorType::Rgb565 => 2,
            ColorType::Rgba8888 => 4,
        }
    }
}
