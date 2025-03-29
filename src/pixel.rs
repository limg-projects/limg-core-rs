pub const RGB_CHANNELS: usize = 3;

pub const PIXEL_BYTES: usize = size_of::<u16>();
pub const PIXEL_R_MASK: u16 = 0xF800;
pub const PIXEL_G_MASK: u16 = 0x07E0;
pub const PIXEL_B_MASK: u16 = 0x001F;

#[inline(always)]
pub const fn rgb_to_pixel(rgb: [u8; RGB_CHANNELS]) -> u16 {
    red_to_pixel(rgb[0]) | green_to_pixel(rgb[1]) | blue_to_pixel(rgb[2])
}

#[inline(always)]
pub const fn red_to_pixel(r: u8) -> u16 {
    ((r as u16) << 8) & PIXEL_R_MASK
}

#[inline(always)]
pub const fn green_to_pixel(g: u8) -> u16 {
    ((g as u16) << 3) & PIXEL_G_MASK
}

#[inline(always)]
pub const fn blue_to_pixel(b: u8) -> u16 {
    ((b as u16) >> 3) & PIXEL_B_MASK
}

#[inline(always)]
pub const fn pixel_to_rgb(pixel: u16) -> [u8; RGB_CHANNELS] {
    [pixel_to_red(pixel), pixel_to_green(pixel), pixel_to_blue(pixel)]
}

#[inline(always)]
pub const fn pixel_to_red(pixel: u16) -> u8 {
    let r = ((pixel & PIXEL_R_MASK) >> 11) as u8;
    (r << 3) | (r >> 2)
}

#[inline(always)]
pub const fn pixel_to_green(pixel: u16) -> u8 {
    let g = ((pixel & PIXEL_G_MASK) >>  5) as u8;
    (g << 2) | (g >> 4)
}

#[inline(always)]
pub const fn pixel_to_blue(pixel: u16) -> u8 {
    let b = (pixel & PIXEL_B_MASK) as u8;
    (b << 3) | (b >> 2)
}