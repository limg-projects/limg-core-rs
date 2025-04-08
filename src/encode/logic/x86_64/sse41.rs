use crate::{pixel::PIXEL_BYTES, ColorType};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use crate::encode::logic::scalar;

const PIXEL_BLOCK_LEN: usize = 8; // u16(16 bit) * 8 = 128 bit

#[cfg(target_arch = "x86")]
use ::core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use ::core::arch::x86_64::*;

#[repr(align(16))]
struct M128I8([i8; 16]);
impl M128I8 {
    #[inline(always)]
    pub const fn as_vector(self) -> __m128i {
        unsafe { ::core::mem::transmute(self.0) }
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn _mm_swap_epi16(a: __m128i) -> __m128i {
    const BYTE_SWAP_MASK: M128I8 = M128I8([1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14]);
    unsafe { _mm_shuffle_epi8(a, BYTE_SWAP_MASK.as_vector()) }
}

#[inline(always)]
#[cfg(target_endian = "big")]
unsafe fn _mm_be_epi16(a: __m128i) -> __m128i {
    a
}

#[inline]
#[cfg(not(target_endian = "big"))]
#[target_feature(enable = "sse4.1")]
unsafe fn _mm_be_epi16(a: __m128i) -> __m128i {
    unsafe { _mm_swap_epi16(a) }
}

#[inline(always)]
#[cfg(target_endian = "little")]
unsafe fn _mm_le_epi16(a: __m128i) -> __m128i {
    a
}

#[inline]
#[cfg(not(target_endian = "little"))]
#[target_feature(enable = "sse4.1")]
unsafe fn _mm_le_epi16(a: __m128i) -> __m128i {
    unsafe { _mm_swap_epi16(a) }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub unsafe fn encode_from_rgb565_swap(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    let mut src_ptr = data.as_ptr();
    let mut dst_ptr = buf.as_mut_ptr();
    
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    for _ in 0..pixel_blocks {
        unsafe {
            let src = _mm_loadu_si128(src_ptr.cast::<__m128i>());
            let swap_src = _mm_swap_epi16(src);

            _mm_storeu_si128(dst_ptr.cast::<__m128i>(), swap_src);
            
            src_ptr = src_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
            dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
        }
    }

    let data = unsafe { from_raw_parts(src_ptr, remainder * PIXEL_BYTES) };
    let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };

    unsafe { scalar::encode_from_rgb565_swap(data, buf, remainder) }
}

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident) => {
        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "sse4.1")]
        pub unsafe fn $rgb888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;
        
            const R_SHUFFLE_MASK_1: M128I8 = M128I8([0, 3, 6, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
            const G_SHUFFLE_MASK_1: M128I8 = M128I8([1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
            const B_SHUFFLE_MASK_1: M128I8 = M128I8([2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        
            const R_SHUFFLE_MASK_2: M128I8 = M128I8([-1, -1, -1, -1, 0, 3, 6, 9, -1, -1, -1, -1, -1, -1, -1, -1]);
            const G_SHUFFLE_MASK_2: M128I8 = M128I8([-1, -1, -1, -1, 1, 4, 7, 10, -1, -1, -1, -1, -1, -1, -1, -1]);
            const B_SHUFFLE_MASK_2: M128I8 = M128I8([-1, -1, -1, -1, 2, 5, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1]);

            // バッファオーバーしないための後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 {
                return unsafe { scalar::$rgb888(data, buf, num_pixels) };
            }
        
            let mut src_ptr = data.as_ptr();
            let mut dst_ptr = buf.as_mut_ptr();
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                unsafe {
                    // 前半4ピクセル取得
                    let rgb_1 = _mm_loadu_si128(src_ptr.cast::<__m128i>());
                    let r_pixel_1 = _mm_shuffle_epi8(rgb_1, R_SHUFFLE_MASK_1.as_vector());
                    let g_pixel_1 = _mm_shuffle_epi8(rgb_1, G_SHUFFLE_MASK_1.as_vector());
                    let b_pixel_1 = _mm_shuffle_epi8(rgb_1, B_SHUFFLE_MASK_1.as_vector());
        
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
        
                    // 後半4ピクセル取得
                    let rgb_2 = _mm_loadu_si128(src_ptr.cast::<__m128i>());
                    let r_pixel_2 = _mm_shuffle_epi8(rgb_2, R_SHUFFLE_MASK_2.as_vector());
                    let g_pixel_2 = _mm_shuffle_epi8(rgb_2, G_SHUFFLE_MASK_2.as_vector());
                    let b_pixel_2 = _mm_shuffle_epi8(rgb_2, B_SHUFFLE_MASK_2.as_vector());
        
                    // 8ピクセルに合成
                    let mut r_pixel = _mm_or_si128(r_pixel_1, r_pixel_2);
                    let mut g_pixel = _mm_or_si128(g_pixel_1, g_pixel_2);
                    let mut b_pixel = _mm_or_si128(b_pixel_1, b_pixel_2);
        
                    // [u8 * 8] -> [u16 * 8]に拡張
                    r_pixel = _mm_cvtepu8_epi16(r_pixel);
                    g_pixel = _mm_cvtepu8_epi16(g_pixel);
                    b_pixel = _mm_cvtepu8_epi16(b_pixel);
        
                    // 右シフトで減色
                    r_pixel = _mm_srli_epi16(r_pixel, 3);
                    g_pixel = _mm_srli_epi16(g_pixel, 2);
                    b_pixel = _mm_srli_epi16(b_pixel, 3);
        
                    // 左シフトで合成位置に移動
                    r_pixel = _mm_slli_epi16(r_pixel, 11);
                    g_pixel = _mm_slli_epi16(g_pixel, 5);
        
                    // ピクセルに合成
                    let pixel = $endian_fn(_mm_or_si128(r_pixel, _mm_or_si128(g_pixel, b_pixel)));
        
                    _mm_storeu_si128(dst_ptr.cast::<__m128i>(), pixel);
                    
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                }
            }
        
            let data = unsafe { from_raw_parts(src_ptr, remainder * COLOR_TYPE.bytes_per_pixel()) };
            let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };
        
            unsafe { scalar::$rgb888(data, buf, remainder) }
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            unsafe { scalar::encode_from_rgb565_direct(data, buf, num_pixels) }
        }
        
        #[inline]
        #[target_feature(enable = "sse4.1")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            unsafe { encode_from_rgb565_swap(data, buf, num_pixels) }
        }

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "sse4.1")]
        pub unsafe fn $rgba8888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;
        
            const R_SHUFFLE_MASK_1: M128I8 = M128I8([0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
            const G_SHUFFLE_MASK_1: M128I8 = M128I8([1, 5, 9, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
            const B_SHUFFLE_MASK_1: M128I8 = M128I8([2, 6, 10, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        
            const R_SHUFFLE_MASK_2: M128I8 = M128I8([-1, -1, -1, -1, 0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1]);
            const G_SHUFFLE_MASK_2: M128I8 = M128I8([-1, -1, -1, -1, 1, 5, 9, 13, -1, -1, -1, -1, -1, -1, -1, -1]);
            const B_SHUFFLE_MASK_2: M128I8 = M128I8([-1, -1, -1, -1, 2, 6, 10, 14, -1, -1, -1, -1, -1, -1, -1, -1]);
        
            let mut src_ptr = data.as_ptr();
            let mut dst_ptr = buf.as_mut_ptr();
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                unsafe {
                    // 前半4ピクセル取得
                    let rgb_1 = _mm_loadu_si128(src_ptr.cast::<__m128i>());
                    let r_pixel_1 = _mm_shuffle_epi8(rgb_1, R_SHUFFLE_MASK_1.as_vector());
                    let g_pixel_1 = _mm_shuffle_epi8(rgb_1, G_SHUFFLE_MASK_1.as_vector());
                    let b_pixel_1 = _mm_shuffle_epi8(rgb_1, B_SHUFFLE_MASK_1.as_vector());
        
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
        
                    // 後半4ピクセル取得
                    let rgb_2 = _mm_loadu_si128(src_ptr.cast::<__m128i>());
                    let r_pixel_2 = _mm_shuffle_epi8(rgb_2, R_SHUFFLE_MASK_2.as_vector());
                    let g_pixel_2 = _mm_shuffle_epi8(rgb_2, G_SHUFFLE_MASK_2.as_vector());
                    let b_pixel_2 = _mm_shuffle_epi8(rgb_2, B_SHUFFLE_MASK_2.as_vector());
        
                    // 8ピクセルに合成
                    let mut r_pixel = _mm_or_si128(r_pixel_1, r_pixel_2);
                    let mut g_pixel = _mm_or_si128(g_pixel_1, g_pixel_2);
                    let mut b_pixel = _mm_or_si128(b_pixel_1, b_pixel_2);
        
                    // [u8 * 8] -> [u16 * 8]に拡張
                    r_pixel = _mm_cvtepu8_epi16(r_pixel);
                    g_pixel = _mm_cvtepu8_epi16(g_pixel);
                    b_pixel = _mm_cvtepu8_epi16(b_pixel);
        
                    // 右シフトで減色
                    r_pixel = _mm_srli_epi16(r_pixel, 3);
                    g_pixel = _mm_srli_epi16(g_pixel, 2);
                    b_pixel = _mm_srli_epi16(b_pixel, 3);
        
                    // 左シフトで合成位置に移動
                    r_pixel = _mm_slli_epi16(r_pixel, 11);
                    g_pixel = _mm_slli_epi16(g_pixel, 5);
        
                    // ピクセルに合成
                    let pixel = $endian_fn(_mm_or_si128(r_pixel, _mm_or_si128(g_pixel, b_pixel)));
        
                    _mm_storeu_si128(dst_ptr.cast::<__m128i>(), pixel);
                    
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                }
            }
        
            let data = unsafe { from_raw_parts(src_ptr, remainder * COLOR_TYPE.bytes_per_pixel()) };
            let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };
        
            unsafe { scalar::$rgba8888(data, buf, remainder) }
        }
    };
}

encode_from_endian!("big", _mm_be_epi16, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", _mm_le_epi16, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);

#[cfg(test)]
mod tests {
    use crate::encode::logic::scalar;
    use crate::PIXEL_BYTES;

    use crate::encode::logic::tests::{NUM_PIXELS, RGB888_DATA, RGB565_DATA, RGBA8888_DATA};

    #[test]
    fn encode_rgb888_x86_64_sse41() {
        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgb888_be(&RGB888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb888_be(&RGB888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgb888_le(&RGB888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb888_le(&RGB888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_rgb565_x86_64() {
        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
        let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

        unsafe {
            scalar::encode_from_rgb565_be(data, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb565_be(data, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgb565_le(data, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb565_le(data, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_rgba8888_x86_64() {
        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgba8888_be(&RGBA8888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgba8888_be(&RGBA8888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgba8888_le(&RGBA8888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgba8888_le(&RGBA8888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_endian_x86_64() {
        let mut a_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut b_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
        let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

        unsafe {
            super::encode_from_rgb888_be(&RGB888_DATA, &mut a_buf, NUM_PIXELS);

            super::encode_from_rgb565_be(data, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_be(&RGBA8888_DATA, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);

            
            super::encode_from_rgb888_le(&RGB888_DATA, &mut a_buf, NUM_PIXELS);

            super::encode_from_rgb565_le(data, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
            super::encode_from_rgba8888_le(&RGBA8888_DATA, &mut b_buf, NUM_PIXELS);
            assert_eq!(a_buf, b_buf);
        }
    }
}