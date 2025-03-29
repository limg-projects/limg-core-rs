/// The 4-byte ASCII file signature used to identify this image format.
pub const IMAGE_SIGNATURE: [u8; 4] = *b"LIMG";

/// The native-endian representation of the file signature, used for fast comparisons.
pub(crate) const IMAGE_SIGNATURE_U32_NE: u32 = u32::from_ne_bytes(IMAGE_SIGNATURE);

/// The size in bytes of the image header.
pub const IMAGE_HEADER_SIZE: usize = size_of::<ImageHeader>();

/// The current format version of the image header.
pub const IMAGE_CURRENT_VARSION: u8 = 1;

/// Bit mask indicating little-endian pixel data layout.
///
/// If this bit is set in the header's `flag` field, pixel data is stored in little-endian format.
/// Otherwise, big-endian is assumed.
pub const IMAGE_FLAG_ENDIAN_BIT: u8 = 0b00000001;


/// The public representation of an image file header.
///
/// This struct matches the binary layout of the on-disk format and is used
/// for serialization and deserialization of image metadata.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageHeader {
    /// 4-byte format signature (should be `"LIMG"`).
    pub signature: [u8; 4],
    /// Format version (currently `1`).
    pub version: u8,
    /// Format flags (e.g., endian encoding).
    pub flag: u8,
    /// Image width in pixels.
    pub width: u16,
    /// Image height in pixels.
    pub height: u16,
    /// Transparent pixel color in 16-bit RGB565 format.
    pub transparent_color: u16,
}

/// Internal header representation used for fast parsing and FFI compatibility.
///
/// In this struct, the signature is stored as a `u32` for fast native-endian comparison.
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
