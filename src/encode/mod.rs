mod logic;

use crate::header::{ImageHeaderInternal, IMAGE_CURRENT_VARSION, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{ColorType, PIXEL_BYTES};
use crate::error::{Error, Result};

use logic::{
    encode_from_rgb888_be,   encode_from_rgb888_le,
    encode_from_rgb565_be,   encode_from_rgb565_le,
    encode_from_rgba8888_be, encode_from_rgba8888_le,
};

/// Calculates the total number of bytes needed to encode an image with the given specification.
///
/// This includes both the image header and the pixel data region.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{ImageSpec, encode_bounds, rgb_to_pixel};
/// 
/// let spec = ImageSpec::new(100, 100, rgb_to_pixel([0, 0, 0]));
/// let bounds = encode_bounds(&spec);
/// 
/// // HeaderSize(12) + width(100) * height(100) * PixelSize(2)
/// assert_eq!(bounds, 20012);
/// ```
#[inline(always)]
pub const fn encoded_size(spec: &ImageSpec) -> usize {
    IMAGE_HEADER_SIZE + spec.num_pixels() * PIXEL_BYTES
}

#[inline]
pub fn encode_to_buffer(data: impl AsRef<[u8]>, buf: &mut impl AsMut<[u8]>, spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let data = data.as_ref();
    encode_args_check(data, spec, color_type)?;

    let buf = buf.as_mut();
    let size = encoded_size(spec);

    if buf.len() < size {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe { encode_image(data, buf, spec, color_type) };

    Ok(size)
}

#[inline(always)]
fn encode_args_check(data: &[u8], spec: &ImageSpec, color_type: ColorType) -> Result<()> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }
    if data.len() < spec.num_pixels() * color_type.bytes_per_pixel() {
        return Err(Error::InputBufferTooSmall);
    }

    Ok(())
}

unsafe fn encode_image(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) {
    let header = ImageHeaderInternal {
        signature: IMAGE_SIGNATURE_U32_NE,
        version: IMAGE_CURRENT_VARSION,
        flag: spec.flag(),
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.unwrap_or(0),
    };

    let header_ptr = (&header as *const ImageHeaderInternal).cast::<u8>();

    unsafe {
        ::core::ptr::copy_nonoverlapping(header_ptr, buf.as_mut_ptr(), IMAGE_HEADER_SIZE);
    }

    let buf = unsafe { buf.get_unchecked_mut(IMAGE_HEADER_SIZE..) };
    let num_pixels = spec.num_pixels();

    unsafe {
        match spec.data_endian {
            DataEndian::Big => {
                match color_type {
                    ColorType::Rgb888 => encode_from_rgb888_be(data, buf, num_pixels),
                    ColorType::Rgb565 => encode_from_rgb565_be(data, buf, num_pixels),
                    ColorType::Rgba8888 => encode_from_rgba8888_be(data, buf, num_pixels),
                }
            },
            DataEndian::Little => {
                match color_type {
                    ColorType::Rgb888 => encode_from_rgb888_le(data, buf, num_pixels),
                    ColorType::Rgb565 => encode_from_rgb565_le(data, buf, num_pixels),
                    ColorType::Rgba8888 => encode_from_rgba8888_le(data, buf, num_pixels),
                }
            },
        }
    }
}
