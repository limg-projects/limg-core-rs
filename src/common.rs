pub const IMAGE_SIGNATURE: [u8; 4] = *b"LIMG";
pub(crate) const IMAGE_SIGNATURE_U32_NE: u32 = u32::from_ne_bytes(IMAGE_SIGNATURE);

pub const IMAGE_HEADER_SIZE: usize = size_of::<ImageHeader>();

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageHeader {
    pub signature: [u8; 4],
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16,
    pub reserve: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImageHeaderInternal {
    pub signature: u32,
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16,
    pub reserve: u16,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageSpec {
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16
}

impl ImageSpec {
    pub(crate) const fn num_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub(crate) const fn is_zero_dimensions(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}
