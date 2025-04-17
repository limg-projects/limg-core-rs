/// Limg画像形式のピクセルエンディアン
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelEndian {
    /// ビッグエンディアン
    Big = 0,
    
    /// リトルエンディアン
    Little = 1,
}

/// Limg画像形式仕様
///
/// この構造体はエンコードおよびデコードで使用されます。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSpec {
    /// 画像の幅
    pub width: u16,

    /// 画像の高さ
    pub height: u16,

    /// 透明色に指定する色
    /// 
    /// 指定しない場合`None`です。
    pub transparent_color: Option<u16>,

    /// 画像のピクセルエンディアン
    pub pixel_endian: PixelEndian,
}

impl ImageSpec {
    /// 画像の幅と高さを指定して`ImageSpec`を作成します。
    /// 
    /// `transparent_color`は`None`になり、`pixel_endian`は`PixelEndian::Little`になります。
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::{ImageSpec, PixelEndian};
    /// 
    /// let spec = ImageSpec::new(100, 100);
    /// 
    /// assert_eq!(spec.transparent_color, None);
    /// assert_eq!(spec.pixel_endian, PixelEndian::Little);
    /// ```
    pub const fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            transparent_color: None,
            pixel_endian: PixelEndian::Little
        }
    }

    /// 画像の幅と高さに透明色を指定して`ImageSpec`を作成します。
    /// 
    /// `pixel_endian`は`PixelEndian::Little`になります。
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::{ImageSpec, PixelEndian, rgb_to_pixel};
    /// 
    /// let spec = ImageSpec::with_transparent_color(100, 100, rgb_to_pixel([255, 255, 255]));
    /// 
    /// assert_eq!(spec.transparent_color, Some(rgb_to_pixel([255, 255, 255])));
    /// assert_eq!(spec.pixel_endian, PixelEndian::Little);
    /// ```
    pub const fn with_transparent_color(width: u16, height: u16, transparent_color: u16) -> Self {
        Self {
            width,
            height,
            transparent_color: Some(transparent_color),
            pixel_endian: PixelEndian::Little
        }
    }

    /// 画像の幅と高さにピクセルエンディアンを指定して`ImageSpec`を作成します。
    /// 
    /// `transparent_color`は`None`になります。
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::{ImageSpec, PixelEndian};
    /// 
    /// let spec = ImageSpec::with_pixel_endian(100, 100, PixelEndian::Big);
    /// 
    /// assert_eq!(spec.transparent_color, None);
    /// assert_eq!(spec.pixel_endian, PixelEndian::Big);
    /// ```
    pub const fn with_pixel_endian(width: u16, height: u16, pixel_endian: PixelEndian) -> Self {
        Self {
            width,
            height,
            transparent_color: None,
            pixel_endian
        }
    }

    /// 合計ピクセル数を返します
    /// 
    /// # Examples
    /// 
    /// ```
    /// use limg_core::ImageSpec;
    /// 
    /// let spec = ImageSpec::new(100, 100);
    /// assert_eq!(spec.num_pixels(), 10000)
    /// ```
    pub const fn num_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }
}
