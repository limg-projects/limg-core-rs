use crate::pixel::{pixel_to_rgb, ColorType, PIXEL_BYTES, PIXEL_R_MASK, PIXEL_G_MASK, PIXEL_B_MASK};
use ::core::ptr::copy_nonoverlapping;
use crate::decode::logic::scalar;

const PIXEL_BLOCK_LEN: usize = 8; // u16(16 bit) * 8 = 128 bit

const R_MASK: i16 = PIXEL_R_MASK as i16;
const G_MASK: i16 = PIXEL_G_MASK as i16;
const B_MASK: i16 = PIXEL_B_MASK as i16;

#[cfg(target_arch = "x86")]
use ::core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use ::core::arch::x86_64::*;

macro_rules! encode_from_endian {
    ($endian: expr, $endian_fn: ident, $rgb888: ident, $rgb565: ident, $rgba8888: ident, $rgba8888_aplha: ident) => {
        // -- rgb888 ------------------------------

        #[inline]
        #[target_feature(enable = "sse4.1")]
        pub unsafe fn $rgb888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;

            // バッファオーバーしないための後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 {
                return unsafe { scalar::$rgb888(data, buf, num_pixels) };
            }
        
            let mut src_ptr = data.as_ptr();
            let mut dst_ptr = buf.as_mut_ptr();
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;

            // let R_PIXEL_MASK = __
        }
    };
}

unsafe fn a(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    const COLOR_TYPE: ColorType = ColorType::Rgb888;

    // バッファオーバーしないための後ピクセルを加味する
    if num_pixels < PIXEL_BLOCK_LEN + 2 {
        return unsafe { scalar::decode_to_rgb888_be(data, buf, num_pixels) };
    }

    let mut src_ptr = data.as_ptr();
    let mut dst_ptr = buf.as_mut_ptr();
    
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    let r_pixel_mask = unsafe { _mm_set_epi16(R_MASK, R_MASK, R_MASK, R_MASK, R_MASK, R_MASK, R_MASK, R_MASK) };
    let g_pixel_mask = unsafe { _mm_set_epi16(G_MASK, G_MASK, G_MASK, G_MASK, G_MASK, G_MASK, G_MASK, G_MASK) };
    let b_pixel_mask = unsafe { _mm_set_epi16(B_MASK, B_MASK, B_MASK, B_MASK, B_MASK, B_MASK, B_MASK, B_MASK) };

    let r_shuffle_mask = unsafe { _mm_set_epi8(0, -1, -1, 2, -1, -1, 4, -1, -1, 6, -1, -1, -1, -1, -1, -1) };
    let g_shuffle_mask = unsafe { _mm_set_epi8(-1, 0, -1, -1, 2, -1, -1, 4, -1, -1, 6, -1, -1, -1, -1, -1) };
    let b_shuffle_mask = unsafe { _mm_set_epi8(-1, -1, 0, -1, -1, 2, -1, -1, 4, -1, -1, 6, -1, -1, -1, -1) };

    for _ in 0..pixel_blocks {
        unsafe {
            // 8ピクセル取得
            let pixel = _mm_loadu_si128(src_ptr.cast::<__m128i>());

            // マスクで色を分離
            let mut r_vec = _mm_and_si128(pixel, r_pixel_mask);
            let mut g_vec = _mm_and_si128(pixel, g_pixel_mask);
            let mut b_vec = _mm_and_si128(pixel, b_pixel_mask);

            r_vec = _mm_srli_epi16(r_vec, 11);
            g_vec = _mm_srli_epi16(g_vec, 5);

            r_vec = _mm_or_si128(_mm_slli_epi16(r_vec, 3), _mm_srli_epi16(r_vec, 2));
            g_vec = _mm_or_si128(_mm_slli_epi16(g_vec, 2), _mm_srli_epi16(g_vec, 4));
            b_vec = _mm_or_si128(_mm_slli_epi16(b_vec, 3), _mm_srli_epi16(b_vec, 2));

            r_vec = _mm_shuffle_epi8(r_vec, r_shuffle_mask);
            g_vec = _mm_shuffle_epi8(g_vec, g_shuffle_mask);
            b_vec = _mm_shuffle_epi8(b_vec, b_shuffle_mask);

            // rgbに合成
            let rgb = _mm_or_si128(r_vec, _mm_or_si128(g_vec, b_vec));

            _mm_storeu_si128(dst_ptr.cast::<__m128i>(), rgb);

            src_ptr = src_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
            dst_ptr = dst_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
        }
    }

    let data = unsafe { ::core::slice::from_raw_parts(src_ptr, remainder * PIXEL_BYTES) };
    let buf = unsafe { ::core::slice::from_raw_parts_mut(dst_ptr, remainder * COLOR_TYPE.bytes_per_pixel()) };
    
    unsafe { scalar::decode_to_rgb888_be(data, buf, remainder) }
}

encode_from_endian!(
    "big",
    to_be,
    decode_to_rgb888_be,
    decode_to_rgb565_be,
    decode_to_rgba8888_be,
    decode_to_rgba8888_alpha_be
);
