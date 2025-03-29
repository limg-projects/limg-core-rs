#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageSpec {
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16
}

impl ImageSpec {
    pub const fn num_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub const fn is_zero_dimensions(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}
