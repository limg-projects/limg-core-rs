use crate::common::header::IMAGE_FLAG_USE_TRANSPARENT_BIT;

/// Represents the byte order (endianness) used for pixel data in the image format.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataEndian {
    /// Big-endian byte order.
    Big = 0,
    /// Little-endian byte order.
    Little = 1,
}

/// Describes the image's specification.
///
/// This struct is used throughout the crate to specify how image data is structured
/// during encoding and decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSpec {
    /// Image width in pixels.
    pub width: u16,
    /// Image height in pixels.
    pub height: u16,
    /// Transparent pixel color in 16-bit RGB565 format.
    pub transparent_color: Option<u16>,
    /// The endianness used for pixel data encoding.
    pub data_endian: DataEndian,
}

impl ImageSpec {
    /// Creates a new `ImageSpec` with little-endian pixel layout by default.
    ///
    /// # Arguments
    ///
    /// * `width` - Image width in pixels.
    /// * `height` - Image height in pixels.
    /// * `transparent_color` - Transparent pixel color in 16-bit RGB565 format.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::spec::{DataEndian, ImageSpec};
    /// 
    /// let spec = ImageSpec::new(100, 100);
    /// assert_eq!(spec.data_endian, DataEndian::Little);
    /// ```
    pub const fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            transparent_color: None,
            data_endian: DataEndian::Little
        }
    }

    pub const fn with_transparent_color(width: u16, height: u16, transparent_color: Option<u16>) -> Self {
        Self {
            width,
            height,
            transparent_color,
            data_endian: DataEndian::Little
        }
    }

    pub const fn with_data_endian(width: u16, height: u16, data_endian: DataEndian) -> Self {
        Self {
            width,
            height,
            transparent_color: None,
            data_endian
        }
    }

    /// 合計ピクセル数を返します
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::spec::ImageSpec;
    /// 
    /// let spec = ImageSpec::new(100, 100);
    /// assert_eq!(spec.num_pixels(), 10000)
    /// ```
    pub const fn num_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// 幅か高さが0の時、trueを返します
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::spec::ImageSpec;
    /// 
    /// let mut spec = ImageSpec::new(100, 100);
    /// assert_eq!(spec.is_zero_dimensions(), false);
    /// spec.width = 0;
    /// assert_eq!(spec.is_zero_dimensions(), true);
    /// ```
    pub const fn is_zero_dimensions(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Returns the internal flag value used to encode the data endianness.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::spec::{DataEndian, ImageSpec};
    /// 
    /// let mut spec = ImageSpec::with_data_endian(100, 100, DataEndian::Little);
    /// 
    /// assert!(spec.flag() != 0);
    /// ```
    pub const fn flag(&self) -> u8 {
        let use_transparent = match self.transparent_color {
            Some(_) => IMAGE_FLAG_USE_TRANSPARENT_BIT,
            None => 0,
        };
        
        (self.data_endian as u8) |
        (use_transparent)
    }
}
