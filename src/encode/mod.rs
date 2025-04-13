//! このモジュールはエンコード関数を提供します。
//! 
//! # Examples
//! 
//! ```
//! use limg_core::encode::{encoded_size, encode, encode_header, encode_data};
//! use limg_core::spec::ImageSpec;
//! use limg_core::pixel::ColorType;
//! 
//! let data = [255, 255, 255];
//! let spec = ImageSpec::new(1, 1);
//! let mut buf_1 = vec![0u8; encoded_size(&spec)];
//! let mut buf_2 = vec![0u8; encoded_size(&spec)];
//! 
//! encode(&data, &mut buf_1, &spec, ColorType::Rgb888).unwrap();
//! 
//! let written_header_size = encode_header(&mut buf_2, &spec).unwrap();
//! encode_data(&data, &mut buf_2[written_header_size..], &spec, ColorType::Rgb888).unwrap();
//! 
//! assert_eq!(buf_1, buf_2);
//! ```

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

/// ImageSpecとColorTypeから、Limg形式データをバッファに書き込みます。
/// 
/// エラーで無かった場合、書き込まれたバイト数を返します。
/// 
/// # Errors
/// spec.widthかspec.heightが0の場合、Error::ZeroImageDimensionsを返します。
/// dataバッファの長さが入力として足りない場合、Error::InputBufferTooSmallを返します。
/// bufバッファの長さがLimg形式データ書き込みサイズに満たない場合、Error::OutputBufferTooSmallを返します。
/// 
/// # Examples
/// 
/// ```
/// use limg_core::encode::{encoded_size, encode};
/// use limg_core::spec::ImageSpec;
/// use limg_core::pixel::ColorType;
/// 
/// fn main() -> limg_core::Result<()> {
///     let data = [255, 255, 255];
///     let spec = ImageSpec::new(1, 1);
///     let mut buf = vec![0u8; encoded_size(&spec)];
/// 
///     let written_size = encode(&data, &mut buf, &spec, ColorType::Rgb888)?;
///     assert_eq!(written_size, encoded_size(&spec));
///     Ok(())
/// }
/// ```
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

/// ImageSpecからヘッダをエンコードし、バッファに書き込みます。
/// 
/// エラーで無かった場合、書き込まれたバイト数を返します。
/// 
/// # Errors
/// 
/// spec.widthかspec.heightが0の場合、Error::ZeroImageDimensionsを返します。
/// bufバッファの長さがヘッダー書き込みサイズに満たない場合、Error::OutputBufferTooSmallを返します。
/// 
/// # Examples
/// 
/// ```
/// use limg_core::HEADER_SIZE;
/// use limg_core::encode::encode_header;
/// use limg_core::spec::ImageSpec;
/// 
/// fn main() -> limg_core::Result<()> {
///     let spec = ImageSpec::new(1, 1);
///     let mut buf = vec![0u8; HEADER_SIZE];
/// 
///     let written_size = encode_header(&mut buf, &spec)?;
///     assert_eq!(written_size, HEADER_SIZE);
///     Ok(())
/// }
/// ```
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

/// ImageSpecとColorTypeから、色データをエンコードしバッファに書き込みます。
/// 
/// エラーで無かった場合、書き込まれたバイト数を返します。
/// 
/// # Errors
/// 
/// dataバッファの長さが入力として足りない場合、Error::InputBufferTooSmallを返します。
/// bufバッファの長さがデータ書き込みサイズに満たない場合、Error::OutputBufferTooSmallを返します。
/// 
/// # Examples
/// 
/// ```
/// use limg_core::encode::encode_data;
/// use limg_core::spec::ImageSpec;
/// use limg_core::pixel::{ColorType, PIXEL_BYTES};
/// 
/// fn main() -> limg_core::Result<()> {
///     let data = [255, 255, 255];
///     let spec = ImageSpec::new(1, 1);
///     let mut buf = vec![0u8; PIXEL_BYTES * spec.num_pixels()];
/// 
///     let written_size = encode_data(&data, &mut buf, &spec, ColorType::Rgb888)?;
///     assert_eq!(written_size, PIXEL_BYTES * spec.num_pixels());
///     Ok(())
/// }
/// ```
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
