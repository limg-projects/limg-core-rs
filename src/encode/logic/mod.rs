mod scalar;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86_64::encode_logic;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use scalar::encode_logic;

macro_rules! encode_logic_fn {
    ($(#[$attr:meta])*) => {

      #[inline(never)]
      $(#[$attr])*
      pub unsafe fn encode_logic(data: *const u8, buf: *mut u8, spec: &crate::common::spec::ImageSpec, color_type: crate::common::color::ColorType) -> usize {
		let num_pixels = spec.num_pixels();

        unsafe {
			match spec.data_endian {
				crate::common::spec::DataEndian::Big => {
					match color_type {
						crate::common::color::ColorType::Rgb888 => encode_from_rgb888_be(data, buf, num_pixels),
						crate::common::color::ColorType::Rgb565 => encode_from_rgb565_be(data, buf, num_pixels),
						crate::common::color::ColorType::Rgba8888 => encode_from_rgba8888_be(data, buf, num_pixels),
					}
				},
				crate::common::spec::DataEndian::Little => {
					match color_type {
						crate::common::color::ColorType::Rgb888 => encode_from_rgb888_le(data, buf, num_pixels),
						crate::common::color::ColorType::Rgb565 => encode_from_rgb565_le(data, buf, num_pixels),
						crate::common::color::ColorType::Rgba8888 => encode_from_rgba8888_le(data, buf, num_pixels),
					}
				},
			}
        }
    
        crate::common::pixel::PIXEL_BYTES * num_pixels
    }
  };
}

pub(crate) use encode_logic_fn;

#[cfg(test)]
mod tests {
    use crate::common::pixel::rgb_to_pixel;

    pub const NUM_PIXELS: usize = 20;

    pub const RGB888_DATA: [u8; 3 * NUM_PIXELS] = [
          0,   0,   0,
        128,   0,   0,
        255,   0,   0,
          0, 128,   0,
          0, 255,   0,
          0,   0, 128,
          0,   0, 255,
        255, 255,   0,
          0, 255, 255,
        255,   0, 255,
          0, 128, 255,
        255, 128,   0,
         10,  10,  10,
         50,  50,  50,
        100, 100, 100,
        128, 128, 128,
        150, 150, 150,
        200, 200, 200,
        250, 250, 250,
        255, 255, 255,
    ];

    pub const RGB565_DATA: [u16; NUM_PIXELS] = [
        rgb_to_pixel([  0,   0,   0]),
        rgb_to_pixel([128,   0,   0]),
        rgb_to_pixel([255,   0,   0]),
        rgb_to_pixel([  0, 128,   0]),
        rgb_to_pixel([  0, 255,   0]),
        rgb_to_pixel([  0,   0, 128]),
        rgb_to_pixel([  0,   0, 255]),
        rgb_to_pixel([255, 255,   0]),
        rgb_to_pixel([  0, 255, 255]),
        rgb_to_pixel([255,   0, 255]),
        rgb_to_pixel([  0, 128, 255]),
        rgb_to_pixel([255, 128,   0]),
        rgb_to_pixel([ 10,  10,  10]),
        rgb_to_pixel([ 50,  50,  50]),
        rgb_to_pixel([100, 100, 100]),
        rgb_to_pixel([128, 128, 128]),
        rgb_to_pixel([150, 150, 150]),
        rgb_to_pixel([200, 200, 200]),
        rgb_to_pixel([250, 250, 250]),
        rgb_to_pixel([255, 255, 255]),
    ];

    pub const RGBA8888_DATA: [u8; 4 * NUM_PIXELS] = [
          0,   0,   0,   0,
        128,   0,   0, 128,
        255,   0,   0, 255,
          0, 128,   0, 128,
          0, 255,   0,   0,
          0,   0, 128, 128,
          0,   0, 255, 255,
        255, 255,   0,   0,
          0, 255, 255, 255,
        255,   0, 255,   0,
          0, 128, 255, 255,
        255, 128,   0,   0,
         10,  10,  10,  10,
         50,  50,  50,  50,
        100, 100, 100, 100,
        128, 128, 128, 128,
        150, 150, 150, 150,
        200, 200, 200, 200,
        250, 250, 250, 250,
        255, 255, 255, 255
    ];
}