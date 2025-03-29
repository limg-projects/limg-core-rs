#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataEndian {
    Big = 0,
    Little = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSpec {
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16,
    pub data_endian: DataEndian,
}

impl ImageSpec {
    pub const fn new(width: u16, height: u16, transparent_color: u16) -> Self {
        Self {
            width,
            height,
            transparent_color,
            data_endian: DataEndian::Little
        }
    }

    pub const fn num_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub const fn is_zero_dimensions(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub const fn flag(&self) -> u8 {
        self.data_endian as u8
    }
}
