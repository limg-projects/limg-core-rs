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
/// 変換時に減色が発生します。
/// 
/// # Examples
/// 
/// ```
/// use limg_core::rgb_to_pixel;
/// 
/// let pixel = rgb_to_pixel([0, 128, 255]);
/// assert_eq!(pixel, 0x041F); // [0, 130, 255]
/// ```
#[inline(always)]
pub const fn rgb_to_pixel(rgb: [u8; 3]) -> u16 {
    (((rgb[0] as u16) << 8) & PIXEL_R_MASK) |
    (((rgb[1] as u16) << 3) & PIXEL_G_MASK) |
    (((rgb[2] as u16) >> 3) & PIXEL_B_MASK)
}

/// RGB565ピクセルから`[R, G, B]`配列に変換します
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel_to_rgb;
/// 
/// let rgb = pixel_to_rgb(0x041F);
/// assert_eq!(rgb, [0, 130, 255]);
/// ```
#[inline(always)]
pub const fn pixel_to_rgb(pixel: u16) -> [u8; 3] {
    let r = ((pixel & PIXEL_R_MASK) >> 11) as u8;
    let g = ((pixel & PIXEL_G_MASK) >>  5) as u8;
    let b =  (pixel & PIXEL_B_MASK)        as u8;
    [(r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2)]
}
