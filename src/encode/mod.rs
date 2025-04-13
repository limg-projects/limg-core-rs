mod logic;

use crate::common::header::{ImageHeader, CURRENT_VARSION, FLAG_USE_TRANSPARENT_BIT, HEADER_SIZE, SIGNATURE_U32_NE};
use crate::spec::ImageSpec;
use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::error::{Error, Result};

/// ImageSpecからエンコードに必要なバイト数を取得します
///
/// このサイズにはヘッダーとデータ部の合計です
/// 
/// # Examples
/// 
/// ```
/// use limg_core::encode::encoded_size;
/// use limg_core::spec::ImageSpec;
/// 
/// let spec = ImageSpec::new(100, 100);
/// let bounds = encoded_size(&spec);
/// 
/// // HeaderSize(12) + width(100) * height(100) * PixelSize(2)
/// assert_eq!(bounds, 20012);
/// ```
#[inline(always)]
pub const fn encoded_size(spec: &ImageSpec) -> usize {
    HEADER_SIZE + spec.num_pixels() * PIXEL_BYTES
}

#[inline]
pub fn encode(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let num_pixels = spec.width as usize * spec.height as usize;

    if num_pixels == 0 {
        return Err(Error::ZeroImageDimensions);
    }
    if data.len() < color_type.bytes_per_pixel() * num_pixels {
        return Err(Error::InputBufferTooSmall);
    }
    if buf.len() < HEADER_SIZE + PIXEL_BYTES * num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    let mut written_size = 0;

    unsafe {
        written_size += encode_header_unchecked(buf.get_unchecked_mut(..HEADER_SIZE), spec);
        written_size += encode_data_unchecked(data, buf.get_unchecked_mut(HEADER_SIZE..), spec, color_type);
    }

    debug_assert_eq!(written_size, encoded_size(spec));

    Ok(written_size)
}

#[inline]
pub fn encode_header(buf: &mut [u8], spec: &ImageSpec) -> Result<usize> {
    let num_pixel = spec.width as usize * spec.height as usize;

    if num_pixel == 0 {
        return Err(Error::ZeroImageDimensions);
    }

    if buf.len() < HEADER_SIZE {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_header_unchecked(buf, spec))
    }
}

unsafe fn encode_header_unchecked(buf: &mut [u8], spec: &ImageSpec) -> usize {
    let use_transparent = match spec.transparent_color {
        Some(_) => FLAG_USE_TRANSPARENT_BIT,
        None => 0,
    };
    
    let flag = (spec.data_endian as u8) |
        (use_transparent);

    let header = ImageHeader {
        signature: SIGNATURE_U32_NE,
        version: CURRENT_VARSION,
        flag,
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.unwrap_or(0).to_le(),
    };

    let header_ptr = buf.as_mut_ptr().cast::<ImageHeader>();

    unsafe { header_ptr.write_unaligned(header); }

    HEADER_SIZE
}

#[inline]
pub fn encode_data(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let num_pixels = spec.num_pixels();

    if data.len() < color_type.bytes_per_pixel() * num_pixels {
        return Err(Error::InputBufferTooSmall);
    }

    if buf.len() < PIXEL_BYTES * num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_unchecked(data, buf, spec, color_type))
    }
}

#[inline(always)]
unsafe fn encode_data_unchecked(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> usize {
    unsafe { logic::encode_logic(data.as_ptr(), buf.as_mut_ptr(), spec, color_type) }
}
