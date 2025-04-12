mod scalar;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86_64::*;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use scalar::*;


#[cfg(test)]
mod tests {
    use crate::pixel::rgb_to_pixel;

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