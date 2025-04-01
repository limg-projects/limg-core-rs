/// The number of color channels in an RGB pixel (red, green, blue).
pub const RGB_CHANNELS: usize = 3;

/// The size of a single encoded pixel in bytes (2 bytes for RGB565).
pub const PIXEL_BYTES: usize = size_of::<u16>();
/// Bit mask for the red component in RGB565 pixel format.
pub const PIXEL_R_MASK: u16 = 0xF800;
/// Bit mask for the green component in RGB565 pixel format.
pub const PIXEL_G_MASK: u16 = 0x07E0;
/// Bit mask for the blue component in RGB565 pixel format.
pub const PIXEL_B_MASK: u16 = 0x001F;

pub enum ColorType {
    Rgb888,
    Rgb565,
    Rgba8888,
}

/// Converts an `[R, G, B]` array into a packed 16-bit RGB565 pixel.
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
pub const fn rgb_to_pixel(rgb: [u8; RGB_CHANNELS]) -> u16 {
    red_to_pixel(rgb[0]) | green_to_pixel(rgb[1]) | blue_to_pixel(rgb[2])
}

/// Encodes an 8-bit red value into its RGB565 position.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::red_to_pixel;
/// 
/// let pixel = red_to_pixel(255);
/// assert_eq!(pixel, 0xF800); // [255, 0, 0]
/// ```
#[inline(always)]
pub const fn red_to_pixel(r: u8) -> u16 {
    ((r as u16) << 8) & PIXEL_R_MASK
}

/// Encodes an 8-bit green value into its RGB565 position.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::green_to_pixel;
/// 
/// let pixel = green_to_pixel(255);
/// assert_eq!(pixel, 0x07E0); // [0, 255, 0]
/// ```
#[inline(always)]
pub const fn green_to_pixel(g: u8) -> u16 {
    ((g as u16) << 3) & PIXEL_G_MASK
}

/// Encodes an 8-bit blue value into its RGB565 position.
///
/// # Examples
/// 
/// ```
/// use limg_core::blue_to_pixel;
/// 
/// let pixel = blue_to_pixel(255);
/// assert_eq!(pixel, 0x001F); // [0, 0, 255]
/// ```
#[inline(always)]
pub const fn blue_to_pixel(b: u8) -> u16 {
    ((b as u16) >> 3) & PIXEL_B_MASK
}



/// Decodes a packed RGB565 pixel into an `[R, G, B]` array.
/// 
/// The result is an approximated 8-bit array value reconstructed.
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
pub const fn pixel_to_rgb(pixel: u16) -> [u8; RGB_CHANNELS] {
    [pixel_to_red(pixel), pixel_to_green(pixel), pixel_to_blue(pixel)]
}

/// Extracts and scales the red component from an RGB565 pixel.
/// 
/// The result is an approximated 8-bit value reconstructed from 5 bits.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel_to_red;
/// 
/// let r = pixel_to_red(0x041F); // [0, 130, 255]
/// assert_eq!(r, 0);
/// ```
#[inline(always)]
pub const fn pixel_to_red(pixel: u16) -> u8 {
    let r = ((pixel & PIXEL_R_MASK) >> 11) as u8;
    (r << 3) | (r >> 2)
}

/// Extracts and scales the green component from an RGB565 pixel.
/// 
/// The result is an approximated 8-bit value reconstructed from 6 bits.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel_to_green;
/// 
/// let g = pixel_to_green(0x041F); // [0, 130, 255]
/// assert_eq!(g, 130);
/// ```
#[inline(always)]
pub const fn pixel_to_green(pixel: u16) -> u8 {
    let g = ((pixel & PIXEL_G_MASK) >>  5) as u8;
    (g << 2) | (g >> 4)
}

/// Extracts and scales the blue component from an RGB565 pixel.
/// 
/// The result is an approximated 8-bit value reconstructed from 5 bits.
/// 
/// # Examples
/// 
/// ```
/// use limg_core::pixel_to_blue;
/// 
/// let b = pixel_to_blue(0x041F); // [0, 130, 255]
/// assert_eq!(b, 255);
/// ```
#[inline(always)]
pub const fn pixel_to_blue(pixel: u16) -> u8 {
    let b = (pixel & PIXEL_B_MASK) as u8;
    (b << 3) | (b >> 2)
}