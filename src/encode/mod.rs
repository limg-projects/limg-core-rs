mod scalar;

use crate::header::{ImageHeaderInternal, IMAGE_CURRENT_VARSION, IMAGE_HEADER_SIZE, IMAGE_SIGNATURE_U32_NE};
use crate::spec::{DataEndian, ImageSpec};
use crate::pixel::{RGB_CHANNELS, PIXEL_BYTES, rgb_to_pixel};
use crate::error::{Error, Result};
use crate::ColorType;
use ::core::slice::from_raw_parts_mut;

use scalar::{
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

/// Encodes RGB byte data into a image buffer (including header and pixel data),
/// returning how many bytes were written.
/// 
/// # Errors
/// 
/// Each call to `encode` may generate an error indicating that the operation could not be completed.
/// If an error is returned then no data were written to buffer.
/// 
/// `consumed_bytes` and `image_buf` remain unchanged.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{Result, ImageSpec, rgb_to_pixel, encode, encode_bounds};
/// 
/// fn main() -> Result<()> {
///     let spec = ImageSpec::new(2, 2, rgb_to_pixel([0, 0, 0]));
///     let rgb_data = [
///         255,   0,   0,
///           0, 255,   0,
///           0,   0, 255,
///         255, 255, 255,
///     ];
///     let mut image_buf = [0u8; 1024];
///     let mut consumed_bytes = 0;
/// 
///     let written_bytes = encode(&rgb_data, &mut image_buf, &spec, Some(&mut consumed_bytes))?;
///     assert_eq!(written_bytes, encode_bounds(&spec));
/// 
///     Ok(())
/// }
/// ```
#[inline]
pub fn encode(rgb_data: &[u8], image_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }
    if rgb_data.len() < spec.num_pixels() * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }
    if image_buf.len() < encoded_size(spec) {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_unchecked(rgb_data, image_buf, spec, consumed_bytes))
    }
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

    unsafe { encode_logic(data, buf, spec, color_type) };

    Ok(size)
}

#[cfg(feature = "alloc")]
#[inline]
pub fn encode_to_vec(data: impl AsRef<[u8]>, spec: &ImageSpec, color_type: ColorType) -> Result<alloc::vec::Vec<u8>> {
    let data = data.as_ref();
    encode_args_check(data, spec, color_type)?;

    let vec = unsafe {
        let size = encoded_size(spec);

        // 未初期化バッファの確保
        let mut buf = alloc::vec::Vec::with_capacity(size);
        buf.set_len(size);

        // バッファに書き込み
        encode_logic(data, &mut buf, spec, color_type);

        buf
    };
    
    Ok(vec)
}

#[cfg(feature = "std")]
#[inline]
pub fn encode_to_write(data: impl AsRef<[u8]>, writer: &mut impl std::io::Write, spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let data = data.as_ref();
    encode_args_check(data, spec, color_type)?;

    let buf = encode_to_vec(data, spec, color_type)?;
    writer.write_all(&buf)?;
    Ok(buf.len())
}

#[cfg(feature = "std")]
#[inline]
pub fn encode_to_file(data: impl AsRef<[u8]>, path: impl AsRef<std::path::Path>, spec: &ImageSpec, color_type: ColorType) -> Result<()> {
    let data = data.as_ref();
    encode_args_check(data, spec, color_type)?;

    let mut file = std::fs::File::create(path)?;
    encode_to_write(data.as_ref(), &mut file, spec, color_type)?;
    Ok(())
}

fn encode_args_check(data: &[u8], spec: &ImageSpec, color_type: ColorType) -> Result<()> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }
    if data.len() < spec.num_pixels() * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }

    Ok(())
}

unsafe fn encode_logic(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) {
    let header = ImageHeaderInternal {
        signature: IMAGE_SIGNATURE_U32_NE,
        version: IMAGE_CURRENT_VARSION,
        flag: spec.flag(),
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.to_le(),
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

/// Encodes RGB byte data into a image buffer (including header and pixel data).
/// returning how many bytes were written, without doing spec and bounds checking.
/// 
/// For a safe alternative see [`encode`]
/// 
/// # Safety
/// 
/// Caller must guarantee that `rgb_data`, `image_buf`, and `spec` are valid and correctly sized.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{ImageSpec, rgb_to_pixel, encode_unchecked, encode_bounds};
/// 
/// fn main() {
///     let spec = ImageSpec::new(2, 2, rgb_to_pixel([0, 0, 0]));
///     let rgb_data = [
///         255,   0,   0,
///           0, 255,   0,
///           0,   0, 255,
///         255, 255, 255,
///     ];
///     let mut image_buf = [0u8; 1024];
///     let mut consumed_bytes = 0;
/// 
///     let written_bytes = unsafe { encode_unchecked(&rgb_data, &mut image_buf, &spec, Some(&mut consumed_bytes)) };
///     assert_eq!(written_bytes, encode_bounds(&spec));
/// 
/// }
/// ```
#[inline]
pub unsafe fn encode_unchecked(rgb_data: &[u8], image_buf: &mut [u8], spec: &ImageSpec, consumed_bytes: Option<&mut usize>) -> usize {
    let mut written_bytes = 0;
    unsafe {
        written_bytes += encode_header_unchecked(image_buf, spec);

        let data_slice = image_buf.get_unchecked_mut(IMAGE_HEADER_SIZE..);
        written_bytes += encode_data_raw_unchecked(rgb_data, data_slice, spec.num_pixels(), spec.data_endian, consumed_bytes);
    }
    written_bytes
}

/// Writes the image header into the given buffer.
///
/// # Errors
/// 
/// `buf` remain unchanged.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{Result, ImageSpec, rgb_to_pixel, encode_header, IMAGE_HEADER_SIZE};
/// 
/// fn main() -> Result<()> {
///     let spec = ImageSpec::new(2, 2, rgb_to_pixel([0, 0, 0]));
///     let mut buf = [0u8; 1024];
/// 
///     let written_bytes = encode_header(&mut buf, &spec)?;
///     assert_eq!(written_bytes, IMAGE_HEADER_SIZE);
/// 
///     Ok(())
/// }
/// ```
#[inline]
pub fn encode_header(buf: &mut [u8], spec: &ImageSpec) -> Result<usize> {
    if spec.is_zero_dimensions() {
        return Err(Error::ZeroImageDimensions);
    }
    if buf.len() < IMAGE_HEADER_SIZE {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_header_unchecked(buf, spec))
    }
}

/// Writes the image header without any bounds checking.
///
/// For a safe alternative see [`encode_header`]
/// 
/// # Safety
/// 
/// `buf` must be at least [`IMAGE_HEADER_SIZE`] bytes long.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{ImageSpec, rgb_to_pixel, encode_header_unchecked, IMAGE_HEADER_SIZE};
/// 
/// fn main() {
///     let spec = ImageSpec::new(2, 2, rgb_to_pixel([0, 0, 0]));
///     let mut buf = [0u8; 1024];
/// 
///     let written_bytes = unsafe { encode_header_unchecked(&mut buf, &spec) };
///     assert_eq!(written_bytes, IMAGE_HEADER_SIZE);
/// }
/// ```
pub unsafe fn encode_header_unchecked(buf: &mut [u8], spec: &ImageSpec) -> usize {
    let header = ImageHeaderInternal {
        signature: IMAGE_SIGNATURE_U32_NE,
        version: IMAGE_CURRENT_VARSION,
        flag: spec.flag(),
        width: spec.width.to_le(),
        height: spec.height.to_le(),
        transparent_color: spec.transparent_color.to_le(),
    };

    let header_ptr = (&header as *const ImageHeaderInternal).cast::<u8>();

    unsafe {
        ::core::ptr::copy_nonoverlapping(header_ptr, buf.as_mut_ptr(), IMAGE_HEADER_SIZE);
    }

    IMAGE_HEADER_SIZE
}


macro_rules! encode_data_endian {
    ($data: ident, $data_unchecked: ident, $data_raw: ident, $data_raw_unchecked: ident, $to_byte_fn: ident) => {
        /// test $data
        /// 
        /// ```
        /// ```
        #[inline]
        pub fn $data(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
            let pixel_buf = unsafe {
                from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
            };
            $data_raw(rgb_data, pixel_buf, num_pixels, consumed_bytes)
        }
        
        #[inline]
        pub unsafe fn $data_unchecked(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
            let pixel_buf = unsafe {
                from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
            };
            unsafe { $data_raw_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes) }
        }

        #[inline]
        pub fn $data_raw(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> Result<usize> {
            if rgb_data.len() < num_pixels * RGB_CHANNELS {
                return Err(Error::InputBufferTooSmall);
            }
            if pixel_buf.len() < num_pixels * PIXEL_BYTES {
                return Err(Error::OutputBufferTooSmall);
            }
        
            unsafe {
                Ok($data_raw_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes))
            }
        }

        pub unsafe fn $data_raw_unchecked(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, consumed_bytes: Option<&mut usize>) -> usize {
            let mut rgb_ptr = rgb_data.as_ptr().cast::<[u8; RGB_CHANNELS]>();
            let mut pixel_ptr = pixel_buf.as_mut_ptr().cast::<[u8; PIXEL_BYTES]>();
        
            for _ in 0..num_pixels {
                unsafe {
                    *pixel_ptr = rgb_to_pixel(*rgb_ptr).$to_byte_fn();
                    rgb_ptr = rgb_ptr.add(1);
                    pixel_ptr = pixel_ptr.add(1);
                }
            }
        
            if let Some(consumed_bytes) = consumed_bytes {
                *consumed_bytes = num_pixels * RGB_CHANNELS;
            }
        
            num_pixels * PIXEL_BYTES
        }
    };
}

/// Encodes RGB data into a `u16` pixel buffer, selecting endianness at runtime.
/// 
/// # Errors
/// 
/// `consumed_bytes` and `pixel_buf` remain unchanged.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{Result, DataEndian, encode_data, PIXEL_BYTES};
/// 
/// fn main() -> Result<()> {
///     // [r, g, b] * 4
///     let rgb_data = [
///         255,   0,   0,
///           0, 255,   0,
///           0,   0, 255,
///         255, 255, 255,
///     ];
/// 
///     let mut pixel_buf = [0u16; 32];
///     let mut consumed_bytes = 0;
/// 
///     let written_bytes = encode_data(&rgb_data, &mut pixel_buf, 4, DataEndian::Little, Some(&mut consumed_bytes))?;
///     assert_eq!(written_bytes, 4 * PIXEL_BYTES);
/// 
///     Ok(())
/// }
/// ```
#[inline]
pub fn encode_data(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    let pixel_buf = unsafe {
        from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
    };
    encode_data_raw(rgb_data, pixel_buf, num_pixels, data_endian, consumed_bytes)
}

/// Encodes data without any safety checks.
/// 
/// For a safe alternative see [`encode_data`]
/// 
/// # Safety
/// 
/// The caller must ensure all input slices are valid and properly sized.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{DataEndian, encode_data_unchecked, PIXEL_BYTES};
/// 
/// fn main() {
///     // [r, g, b] * 4
///     let rgb_data = [
///         255,   0,   0,
///           0, 255,   0,
///           0,   0, 255,
///         255, 255, 255,
///     ];
/// 
///     let mut pixel_buf = [0u16; 32];
///     let mut consumed_bytes = 0;
/// 
///     let written_bytes = unsafe { encode_data_unchecked(&rgb_data, &mut pixel_buf, 4, DataEndian::Little, Some(&mut consumed_bytes)) };
///     assert_eq!(written_bytes, 4 * PIXEL_BYTES);
/// }
/// ```
#[inline]
pub unsafe fn encode_data_unchecked(rgb_data: &[u8], pixel_buf: &mut [u16], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> usize {
    let pixel_buf = unsafe {
        from_raw_parts_mut(pixel_buf.as_mut_ptr().cast::<u8>(), pixel_buf.len() * PIXEL_BYTES)
    };
    unsafe { encode_data_raw_unchecked(rgb_data, pixel_buf, num_pixels, data_endian, consumed_bytes) }
}

/// Encodes RGB data directly into a raw byte buffer with the given endianness.
/// 
/// # Errors
/// 
/// `consumed_bytes` and `pixel_buf` remain unchanged.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{Result, DataEndian, encode_data_raw, PIXEL_BYTES};
/// 
/// fn main() -> Result<()> {
///     // [r, g, b] * 4
///     let rgb_data = [
///         255,   0,   0,
///           0, 255,   0,
///           0,   0, 255,
///         255, 255, 255,
///     ];
/// 
///     let mut pixel_buf = [0u8; 64];
///     let mut consumed_bytes = 0;
/// 
///     let written_bytes = encode_data_raw(&rgb_data, &mut pixel_buf, 4, DataEndian::Little, Some(&mut consumed_bytes))?;
///     assert_eq!(written_bytes, 4 * PIXEL_BYTES);
/// 
///     Ok(())
/// }
/// ```
#[inline]
pub fn encode_data_raw(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> Result<usize> {
    if rgb_data.len() < num_pixels * RGB_CHANNELS {
        return Err(Error::InputBufferTooSmall);
    }
    if pixel_buf.len() < num_pixels * PIXEL_BYTES {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe {
        Ok(encode_data_raw_unchecked(rgb_data, pixel_buf, num_pixels, data_endian, consumed_bytes))
    }
}

/// Performs raw pixel encoding without any validation checks.
/// 
/// # Safety
/// 
/// The caller must ensure all input slices are valid and properly sized.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{DataEndian, encode_data_raw_unchecked, PIXEL_BYTES};
/// 
/// fn main() {
///     // [r, g, b] * 4
///     let rgb_data = [
///         255,   0,   0,
///           0, 255,   0,
///           0,   0, 255,
///         255, 255, 255,
///     ];
/// 
///     let mut pixel_buf = [0u8; 64];
///     let mut consumed_bytes = 0;
/// 
///     let written_bytes = unsafe { encode_data_raw_unchecked(&rgb_data, &mut pixel_buf, 4, DataEndian::Little, Some(&mut consumed_bytes)) };
///     assert_eq!(written_bytes, 4 * PIXEL_BYTES);
/// }
/// ```
#[inline]
pub unsafe fn encode_data_raw_unchecked(rgb_data: &[u8], pixel_buf: &mut [u8], num_pixels: usize, data_endian: DataEndian, consumed_bytes: Option<&mut usize>) -> usize {
    match data_endian {
        DataEndian::Big => unsafe { encode_data_raw_be_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes) },
        DataEndian::Little => unsafe { encode_data_raw_le_unchecked(rgb_data, pixel_buf, num_pixels, consumed_bytes) },
    }
}

encode_data_endian! {
    encode_data_be,
    encode_data_be_unchecked,
    encode_data_raw_be,
    encode_data_raw_be_unchecked,
    to_be_bytes
}

encode_data_endian! {
    encode_data_le,
    encode_data_le_unchecked,
    encode_data_raw_le,
    encode_data_raw_le_unchecked,
    to_le_bytes
}