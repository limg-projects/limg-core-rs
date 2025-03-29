pub const IMAGE_SIGNATURE: [u8; 4] = *b"LIMG";
pub(crate) const IMAGE_SIGNATURE_U32_NE: u32 = u32::from_ne_bytes(IMAGE_SIGNATURE);

pub const IMAGE_HEADER_SIZE: usize = size_of::<ImageHeader>();

pub const IMAGE_CURRENT_VARSION: u8 = 1;

pub const IMAGE_FLAG_ENDIAN_BIT: u8 = 0x01;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageHeader {
    pub signature: [u8; 4],
    pub version: u8,
    pub flag: u8,
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ImageHeaderInternal {
    pub signature: u32,
    pub version: u8,
    pub flag: u8,
    pub width: u16,
    pub height: u16,
    pub transparent_color: u16,
}
