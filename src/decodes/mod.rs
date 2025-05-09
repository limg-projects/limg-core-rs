mod logic;

use crate::common::color::ColorType;
use crate::common::header::{ImageHeader, FLAG_ENDIAN_BIT, FLAG_USE_TRANSPARENT_BIT, HEADER_SIZE, SIGNATURE_U32_NE};
use crate::common::spec::ImageSpec;
use crate::common::pixel::PIXEL_BYTES;
use crate::error::{Error, Result};

/// `spec`と`color_type`からデコードに必要なバイト数を取得します。
/// 
/// サイズは（色バイト数 * 総ピクセル数）です。
/// 
/// # Examples
/// 
/// ```
/// use limg_core::{ColorType, ImageSpec, decoded_size};
/// 
/// let spec = ImageSpec::new(100, 100);
/// let color_type = ColorType::Rgb888;
/// let size = decoded_size(&spec, color_type);
/// 
/// // width(100) * height(100) * color_bytes(3)
/// assert_eq!(size, 30000);
/// ```
#[inline(always)]
pub const fn decoded_size(spec: &ImageSpec, color_type: ColorType) -> usize {
    color_type.bytes_per_pixel() * spec.num_pixels()
}

/// `data`と`color_type`からLimg形式データをデコードし、`buf`バッファに書き込みます。
/// 
/// エラーではなかった場合、`ImageSpec`と書き込まれたバイト数を返します。
/// 
/// # Errors
/// 
/// `data`の長さが（[`HEADER_SIZE`] + [`PIXEL_BYTES`] * 総ピクセル数）未満の場合、`Error::InputBufferTooSmall`を返します。
/// 
/// ヘッダが不正なデータだった場合、`Error::UnsupportedFormat`を返します。
/// 
/// `buf`の長さが（色バイト数 * 総ピクセル数）未満の場合、`Error::OutputBufferTooSmall`を返します。
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use limg_core::decode;
/// # use limg_core::ColorType;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = std::fs::read("image.limg")?;
/// # let mut buf = vec![0u8; 0];
/// # let color_type = ColorType::Rgb888;
/// let (spec, written_size) = decode(&data, &mut buf, color_type)?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn decode(data: &[u8], buf: &mut [u8], color_type: ColorType) -> Result<(ImageSpec, usize)> {
    let spec = decode_header(data)?;

    let data = unsafe { data.get_unchecked(HEADER_SIZE..) };
    let written_size = decode_data(data, buf, &spec, color_type)?;

    Ok((spec, written_size))
}

/// `data`からヘッダをデコードし、`ImageSpec`を取得します。
/// 
/// エラーではなかった場合、`ImageSpec`を返します。
/// 
/// # Errors
/// 
/// `data`の長さが[`HEADER_SIZE`]未満の場合、`Error::InputBuffferTooSmall`を返します。
/// 
/// ヘッダが不正なデータだった場合、`Error::UnsupportedFormat`を返します。
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use limg_core::decode_header;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = std::fs::read("image.limg")?;
/// let spec = decode_header(&data)?;
/// # Ok(())
/// # }
/// ```
/// 
pub fn decode_header(data: &[u8]) -> Result<ImageSpec> {
    if data.len() < HEADER_SIZE {
        return Err(Error::InputBufferTooSmall);
    }

    let header_ptr = data.as_ptr().cast::<ImageHeader>();
    let header = unsafe { header_ptr.read_unaligned() };

    if header.signature != SIGNATURE_U32_NE {
        return Err(Error::UnsupportedFormat);
    }

    // エンディアン関係なく0はチェック可能
    if header.width == 0 || header.height == 0 {
        return Err(Error::UnsupportedFormat);
    }

    let transparent_color = if (header.flag & FLAG_USE_TRANSPARENT_BIT) != 0 { Some(header.transparent_color) } else { None };
    let pixel_endian = unsafe { ::core::mem::transmute(header.flag & FLAG_ENDIAN_BIT) };

    let spec = ImageSpec {
        width: u16::from_le(header.width),
        height: u16::from_le(header.height),
        transparent_color,
        pixel_endian
    };

    Ok(spec)
}

/// `data`と`spec`、`color_type`から色データをデコードし、`buf`バッファに書き込みます。
/// 
/// エラーではなかった場合、書き込まれたバイト数を返します。
/// 
/// # Errors
/// 
/// `data`の長さが（[`PIXEL_BYTES`] * 総ピクセル数）未満の場合、`Error::InputBufferTooSmall`を返します。
/// 
/// `buf`の長さが（色バイト数 * 総ピクセル数）未満の場合、`Error::OutputBufferTooSmall`を返します。
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use limg_core::{decode_header, decode_data, decoded_size, HEADER_SIZE};
/// # use limg_core::ColorType;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = std::fs::read("image.limg")?;
/// let spec = decode_header(&data)?;
/// 
/// # let color_type = ColorType::Rgb888;
/// let mut buf = vec![0u8; decoded_size(&spec, color_type)];
/// decode_data(&data[HEADER_SIZE..], &mut buf, &spec, color_type)?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn decode_data(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> Result<usize> {
    let num_pixels = spec.num_pixels();

    if data.len() < PIXEL_BYTES * num_pixels {
        return Err(Error::InputBufferTooSmall);
    }

    if buf.len() < color_type.bytes_per_pixel() * num_pixels {
        return Err(Error::OutputBufferTooSmall);
    }

    unsafe { Ok(decode_data_unchecked(data, buf, spec, color_type)) }
}

#[inline(always)]
unsafe fn decode_data_unchecked(data: &[u8], buf: &mut [u8], spec: &ImageSpec, color_type: ColorType) -> usize {
    unsafe { logic::decode_logic(data.as_ptr(), buf.as_mut_ptr(), spec, color_type) }
}
