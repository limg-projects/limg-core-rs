/// エンコードされたピクセルのバイト数
pub const PIXEL_BYTES: usize = 2;
/// RGB565のR情報マスク
pub const PIXEL_R_MASK: u16 = 0xF800;
/// RGB565のG情報マスク
pub const PIXEL_G_MASK: u16 = 0x07E0;
/// RGB565のB情報マスク
pub const PIXEL_B_MASK: u16 = 0x001F;

/// `[R, G, B]`配列からRGB565ピクセルに変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::rgb_to_pixel;
/// 
/// let pixel = rgb_to_pixel([0, 128, 255]);
/// assert_eq!(pixel, 0x041F); // [0, 130, 255]
/// ```
#[inline(always)]
pub const fn rgb_to_pixel(rgb: [u8; 3]) -> u16 {
    red_to_pixel(rgb[0]) | green_to_pixel(rgb[1]) | blue_to_pixel(rgb[2])
}

/// R単色からRGB565ピクセルに変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::red_to_pixel;
/// 
/// let pixel = red_to_pixel(255);
/// assert_eq!(pixel, 0xF800); // [255, 0, 0]
/// ```
#[inline(always)]
pub const fn red_to_pixel(r: u8) -> u16 {
    ((r as u16) << 8) & PIXEL_R_MASK
}

/// G単色からRGB565ピクセルに変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::green_to_pixel;
/// 
/// let pixel = green_to_pixel(255);
/// assert_eq!(pixel, 0x07E0); // [0, 255, 0]
/// ```
#[inline(always)]
pub const fn green_to_pixel(g: u8) -> u16 {
    ((g as u16) << 3) & PIXEL_G_MASK
}

/// B単色からRGB565ピクセルに変換します
///
/// # Examples
/// 
/// ```
/// use limg_core::pixel::blue_to_pixel;
/// 
/// let pixel = blue_to_pixel(255);
/// assert_eq!(pixel, 0x001F); // [0, 0, 255]
/// ```
#[inline(always)]
pub const fn blue_to_pixel(b: u8) -> u16 {
    ((b as u16) >> 3) & PIXEL_B_MASK
}



/// RGB565ピクセルから`[R, G, B]`配列に変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::pixel_to_rgb;
/// 
/// let rgb = pixel_to_rgb(0x041F);
/// assert_eq!(rgb, [0, 130, 255]);
/// ```
#[inline(always)]
pub const fn pixel_to_rgb(pixel: u16) -> [u8; 3] {
    [pixel_to_red(pixel), pixel_to_green(pixel), pixel_to_blue(pixel)]
}

/// RGB565ピクセルからR単色のみ取得して変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::pixel_to_red;
/// 
/// let r = pixel_to_red(0x041F); // [0, 130, 255]
/// assert_eq!(r, 0);
/// ```
#[inline(always)]
pub const fn pixel_to_red(pixel: u16) -> u8 {
    let r = ((pixel & PIXEL_R_MASK) >> 11) as u8;
    (r << 3) | (r >> 2)
}

/// RGB565ピクセルからG単色のみ取得して変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::pixel_to_green;
/// 
/// let g = pixel_to_green(0x041F); // [0, 130, 255]
/// assert_eq!(g, 130);
/// ```
#[inline(always)]
pub const fn pixel_to_green(pixel: u16) -> u8 {
    let g = ((pixel & PIXEL_G_MASK) >>  5) as u8;
    (g << 2) | (g >> 4)
}

/// RGB565ピクセルからB単色のみ取得して変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel::pixel_to_blue;
/// 
/// let b = pixel_to_blue(0x041F); // [0, 130, 255]
/// assert_eq!(b, 255);
/// ```
#[inline(always)]
pub const fn pixel_to_blue(pixel: u16) -> u8 {
    let b = (pixel & PIXEL_B_MASK) as u8;
    (b << 3) | (b >> 2)
}