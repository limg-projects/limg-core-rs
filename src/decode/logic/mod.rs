mod scalar;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86_64::decode_logic;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use scalar::decode_logic;

macro_rules! decode_logic_fn {
    ($(#[$attr:meta])*) => {

        #[inline(never)]
        $(#[$attr])*
        pub unsafe fn decode_logic(data: *const u8, buf: *mut u8, spec: &crate::common::spec::ImageSpec, color_type: crate::common::color::ColorType) -> usize {
            let num_pixels = spec.num_pixels();
        
            unsafe {
                match spec.data_endian {
                    crate::common::spec::DataEndian::Big => {
                        match color_type {
                            crate::common::color::ColorType::Rgb888 => decode_to_rgb888_be(data, buf, num_pixels),
                            crate::common::color::ColorType::Rgb565 => decode_to_rgb565_be(data, buf, num_pixels),
                            crate::common::color::ColorType::Rgba8888 => {
                                if let Some(transparent_color) = spec.transparent_color {
                                    decode_to_rgba8888_alpha_be(data, buf, transparent_color, num_pixels)
                                } else {
                                    decode_to_rgba8888_be(data, buf, num_pixels)
                                }
                            }
                        }
                    },
                    crate::common::spec::DataEndian::Little => {
                        match color_type {
                            crate::common::color::ColorType::Rgb888 => decode_to_rgb888_le(data, buf, num_pixels),
                            crate::common::color::ColorType::Rgb565 => decode_to_rgb565_le(data, buf, num_pixels),
                            crate::common::color::ColorType::Rgba8888 => {
                                if let Some(transparent_color) = spec.transparent_color {
                                    decode_to_rgba8888_alpha_le(data, buf, transparent_color, num_pixels)
                                } else {
                                    decode_to_rgba8888_le(data, buf, num_pixels)
                                }
                            }
                        }
                    },
                }
            }
            
            color_type.bytes_per_pixel() * num_pixels
        }
    };
}

pub(crate) use decode_logic_fn;

#[cfg(test)]
mod tests {
    use crate::common::pixel::rgb_to_pixel;

    pub const NUM_PIXELS: usize = 20;

    pub const RGB565_DATA_BE: [u16; NUM_PIXELS] = [
        rgb_to_pixel([  0,   0,   0]).to_be(),
        rgb_to_pixel([128,   0,   0]).to_be(),
        rgb_to_pixel([255,   0,   0]).to_be(),
        rgb_to_pixel([  0, 128,   0]).to_be(),
        rgb_to_pixel([  0, 255,   0]).to_be(),
        rgb_to_pixel([  0,   0, 128]).to_be(),
        rgb_to_pixel([  0,   0, 255]).to_be(),
        rgb_to_pixel([255, 255,   0]).to_be(),
        rgb_to_pixel([  0, 255, 255]).to_be(),
        rgb_to_pixel([255,   0, 255]).to_be(),
        rgb_to_pixel([  0, 128, 255]).to_be(),
        rgb_to_pixel([255, 128,   0]).to_be(),
        rgb_to_pixel([ 10,  10,  10]).to_be(),
        rgb_to_pixel([ 50,  50,  50]).to_be(),
        rgb_to_pixel([100, 100, 100]).to_be(),
        rgb_to_pixel([128, 128, 128]).to_be(),
        rgb_to_pixel([150, 150, 150]).to_be(),
        rgb_to_pixel([200, 200, 200]).to_be(),
        rgb_to_pixel([250, 250, 250]).to_be(),
        rgb_to_pixel([255, 255, 255]).to_be(),
    ];

    pub const RGB565_DATA_LE: [u16; NUM_PIXELS] = [
        rgb_to_pixel([  0,   0,   0]).to_le(),
        rgb_to_pixel([128,   0,   0]).to_le(),
        rgb_to_pixel([255,   0,   0]).to_le(),
        rgb_to_pixel([  0, 128,   0]).to_le(),
        rgb_to_pixel([  0, 255,   0]).to_le(),
        rgb_to_pixel([  0,   0, 128]).to_le(),
        rgb_to_pixel([  0,   0, 255]).to_le(),
        rgb_to_pixel([255, 255,   0]).to_le(),
        rgb_to_pixel([  0, 255, 255]).to_le(),
        rgb_to_pixel([255,   0, 255]).to_le(),
        rgb_to_pixel([  0, 128, 255]).to_le(),
        rgb_to_pixel([255, 128,   0]).to_le(),
        rgb_to_pixel([ 10,  10,  10]).to_le(),
        rgb_to_pixel([ 50,  50,  50]).to_le(),
        rgb_to_pixel([100, 100, 100]).to_le(),
        rgb_to_pixel([128, 128, 128]).to_le(),
        rgb_to_pixel([150, 150, 150]).to_le(),
        rgb_to_pixel([200, 200, 200]).to_le(),
        rgb_to_pixel([250, 250, 250]).to_le(),
        rgb_to_pixel([255, 255, 255]).to_le(),
    ];
}