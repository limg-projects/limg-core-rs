use crate::pixel::{ColorType, PIXEL_BYTES};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use crate::encode::logic::scalar;

const PIXEL_BLOCK_LEN: usize = 16; // u16(16 bit) * 16 = 256 bit

#[cfg(target_arch = "x86")]
use ::core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use ::core::arch::x86_64::*;

#[repr(align(32))]
struct M256I8([i8; 32]);
impl M256I8 {
    #[inline(always)]
    pub const fn as_vector(self) -> __m256i {
        unsafe { ::core::mem::transmute(self.0) }
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn _mm256_swap_epi16(a: __m256i) -> __m256i {
    const BYTE_SWAP_MASK: M256I8 = M256I8([
        1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14,
        1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14
    ]);
    unsafe { _mm256_shuffle_epi8(a, BYTE_SWAP_MASK.as_vector()) }
}

#[inline(always)]
#[cfg(target_endian = "big")]
unsafe fn _mm256_be_epi16(a: __m256i) -> __m256i {
    a
}

#[inline]
#[cfg(not(target_endian = "big"))]
#[target_feature(enable = "avx2")]
unsafe fn _mm256_be_epi16(a: __m256i) -> __m256i {
    unsafe { _mm256_swap_epi16(a) }
}

#[inline(always)]
#[cfg(target_endian = "little")]
unsafe fn _mm256_le_epi16(a: __m256i) -> __m256i {
    a
}

#[inline]
#[cfg(not(target_endian = "little"))]
#[target_feature(enable = "avx2")]
unsafe fn _mm256_le_epi16(a: __m256i) -> __m256i {
    unsafe { _mm256_swap_epi16(a) }
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn encode_from_rgb565_swap(data: &[u8], buf: &mut [u8], num_pixels: usize) {
    let mut src_ptr = data.as_ptr();
    let mut dst_ptr = buf.as_mut_ptr();
    
    let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
    let remainder = num_pixels % PIXEL_BLOCK_LEN;

    for _ in 0..pixel_blocks {
        unsafe {
            let src = _mm256_loadu_si256(src_ptr.cast::<__m256i>());
            let swap_src = _mm256_swap_epi16(src);

            _mm256_storeu_si256(dst_ptr.cast::<__m256i>(), swap_src);
            
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

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgb888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgb888;
        
            const R_SHUFFLE_MASK_1: M256I8 = M256I8([
                4, 7, 10, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                0, 3,  6,  9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            ]);
            const G_SHUFFLE_MASK_1: M256I8 = M256I8([
                5, 8, 11, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                1, 4,  7, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
            const B_SHUFFLE_MASK_1: M256I8 = M256I8([
                6, 9, 12, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                2, 5,  8, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
        
            const R_SHUFFLE_MASK_2: M256I8 = M256I8([
                -1, -1, -1, -1, 4, 7, 10, 13, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, 0, 3,  6,  9, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
            const G_SHUFFLE_MASK_2: M256I8 = M256I8([
                -1, -1, -1, -1, 5, 8, 11, 14, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, 1, 3,  7, 10, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
            const B_SHUFFLE_MASK_2: M256I8 = M256I8([
                -1, -1, -1, -1, 6, 9, 12, 15, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, 2, 5,  8, 11, -1, -1, -1, -1, -1, -1, -1, -1
            ]);

            // バッファオーバーしないための前後ピクセルを加味する
            if num_pixels < PIXEL_BLOCK_LEN + 2 + 2 {
                return unsafe { scalar::$rgb888(data, buf, num_pixels) };
            }

            // 先頭の2ピクセル先に処理する
            unsafe { scalar::$rgb888(data, buf, 2); }

            let pixel_move_mask = unsafe { _mm256_setr_epi32(0, 4, 1, 5, -1, -1, -1, -1) };

            // 後半処理のため2バイトずらす
            let mut src_ptr = unsafe { data.as_ptr().add(2) };
            // 2ピクセル部分進めておく
            let mut dst_ptr = unsafe { buf.as_mut_ptr().add(4) };

            
            let pixel_blocks = (num_pixels - 2) / PIXEL_BLOCK_LEN;
            let remainder = PIXEL_BLOCK_LEN - (COLOR_TYPE.bytes_per_pixel() * pixel_blocks);
        
            for _ in 0..pixel_blocks {
                unsafe {
                    // 前半8ピクセル取得
                    let rgb_1 = _mm256_loadu_si256(src_ptr.cast::<__m256i>());
                    let r_pixel_1 = _mm256_shuffle_epi8(rgb_1, R_SHUFFLE_MASK_1.as_vector());
                    let g_pixel_1 = _mm256_shuffle_epi8(rgb_1, G_SHUFFLE_MASK_1.as_vector());
                    let b_pixel_1 = _mm256_shuffle_epi8(rgb_1, B_SHUFFLE_MASK_1.as_vector());
        
                    src_ptr = src_ptr.add(8 * COLOR_TYPE.bytes_per_pixel());
        
                    // 後半8ピクセル取得
                    let rgb_2 = _mm256_loadu_si256(src_ptr.cast::<__m256i>());
                    let r_pixel_2 = _mm256_shuffle_epi8(rgb_2, R_SHUFFLE_MASK_2.as_vector());
                    let g_pixel_2 = _mm256_shuffle_epi8(rgb_2, G_SHUFFLE_MASK_2.as_vector());
                    let b_pixel_2 = _mm256_shuffle_epi8(rgb_2, B_SHUFFLE_MASK_2.as_vector());
        
                    // 16ピクセルに合成
                    let mut r_pixel = _mm256_or_si256(r_pixel_1, r_pixel_2);
                    let mut g_pixel = _mm256_or_si256(g_pixel_1, g_pixel_2);
                    let mut b_pixel = _mm256_or_si256(b_pixel_1, b_pixel_2);

                    // 前半16バイトに移動
                    r_pixel = _mm256_permutevar8x32_epi32(r_pixel, pixel_move_mask);
                    g_pixel = _mm256_permutevar8x32_epi32(g_pixel, pixel_move_mask);
                    b_pixel = _mm256_permutevar8x32_epi32(b_pixel, pixel_move_mask);

        
                    // [u8 * 16] -> [u16 * 16]に拡張
                    r_pixel = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(r_pixel));
                    g_pixel = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(g_pixel));
                    b_pixel = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(b_pixel));
        
                    // 右シフトで減色
                    r_pixel = _mm256_srli_epi16(r_pixel, 3);
                    g_pixel = _mm256_srli_epi16(g_pixel, 2);
                    b_pixel = _mm256_srli_epi16(b_pixel, 3);
        
                    // 左シフトで合成位置に移動
                    r_pixel = _mm256_slli_epi16(r_pixel, 11);
                    g_pixel = _mm256_slli_epi16(g_pixel, 5);
        
                    // ピクセルに合成
                    let pixel = $endian_fn(_mm256_or_si256(r_pixel, _mm256_or_si256(g_pixel, b_pixel)));
        
                    _mm256_storeu_si256(dst_ptr.cast::<__m256i>(), pixel);
                    
                    src_ptr = src_ptr.add(4 * COLOR_TYPE.bytes_per_pixel());
                    dst_ptr = dst_ptr.add(PIXEL_BLOCK_LEN * PIXEL_BYTES);
                }
            }
        
            let data = unsafe { from_raw_parts(src_ptr.add(4), remainder * COLOR_TYPE.bytes_per_pixel()) };
            let buf = unsafe { from_raw_parts_mut(dst_ptr, remainder * PIXEL_BYTES) };
        
            unsafe { scalar::$rgba8888(data, buf, remainder) }
        }

        // -- rgb565 ------------------------------

        #[inline(always)]
        #[cfg(target_endian = $endian)]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            unsafe { scalar::encode_from_rgb565_direct(data, buf, num_pixels) }
        }
        
        #[inline]
        #[target_feature(enable = "avx2")]
        #[cfg(not(target_endian = $endian))]
        pub unsafe fn $rgb565(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            unsafe { encode_from_rgb565_swap(data, buf, num_pixels) }
        }

        // -- rgba8888 ----------------------------

        #[inline]
        #[target_feature(enable = "avx2")]
        pub unsafe fn $rgba8888(data: &[u8], buf: &mut [u8], num_pixels: usize) {
            const COLOR_TYPE: ColorType = ColorType::Rgba8888;
        
            const R_SHUFFLE_MASK_1: M256I8 = M256I8([
                0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            ]);
            const G_SHUFFLE_MASK_1: M256I8 = M256I8([
                1, 5, 9, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                1, 5, 9, 13, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
            const B_SHUFFLE_MASK_1: M256I8 = M256I8([
                2, 6, 10, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                2, 6, 10, 14, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
        
            const R_SHUFFLE_MASK_2: M256I8 = M256I8([
                -1, -1, -1, -1, 0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, 0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
            const G_SHUFFLE_MASK_2: M256I8 = M256I8([
                -1, -1, -1, -1, 1, 5, 9, 13, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, 1, 5, 9, 13, -1, -1, -1, -1, -1, -1, -1, -1
            ]);
            const B_SHUFFLE_MASK_2: M256I8 = M256I8([
                -1, -1, -1, -1, 2, 6, 10, 14, -1, -1, -1, -1, -1, -1, -1, -1,
                -1, -1, -1, -1, 2, 6, 10, 14, -1, -1, -1, -1, -1, -1, -1, -1
            ]);

            let pixel_move_mask = unsafe { _mm256_setr_epi32(0, 4, 1, 5, -1, -1, -1, -1) };
        
            let mut src_ptr = data.as_ptr();
            let mut dst_ptr = buf.as_mut_ptr();
            
            let pixel_blocks = num_pixels / PIXEL_BLOCK_LEN;
            let remainder = num_pixels % PIXEL_BLOCK_LEN;
        
            for _ in 0..pixel_blocks {
                unsafe {
                    // 前半8ピクセル取得
                    let rgb_1 = _mm256_loadu_si256(src_ptr.cast::<__m256i>());
                    let r_pixel_1 = _mm256_shuffle_epi8(rgb_1, R_SHUFFLE_MASK_1.as_vector());
                    let g_pixel_1 = _mm256_shuffle_epi8(rgb_1, G_SHUFFLE_MASK_1.as_vector());
                    let b_pixel_1 = _mm256_shuffle_epi8(rgb_1, B_SHUFFLE_MASK_1.as_vector());
        
                    src_ptr = src_ptr.add(8 * COLOR_TYPE.bytes_per_pixel());
        
                    // 後半8ピクセル取得
                    let rgb_2 = _mm256_loadu_si256(src_ptr.cast::<__m256i>());
                    let r_pixel_2 = _mm256_shuffle_epi8(rgb_2, R_SHUFFLE_MASK_2.as_vector());
                    let g_pixel_2 = _mm256_shuffle_epi8(rgb_2, G_SHUFFLE_MASK_2.as_vector());
                    let b_pixel_2 = _mm256_shuffle_epi8(rgb_2, B_SHUFFLE_MASK_2.as_vector());
        
                    // 16ピクセルに合成
                    let mut r_pixel = _mm256_or_si256(r_pixel_1, r_pixel_2);
                    let mut g_pixel = _mm256_or_si256(g_pixel_1, g_pixel_2);
                    let mut b_pixel = _mm256_or_si256(b_pixel_1, b_pixel_2);

                    // 前半16バイトに移動
                    r_pixel = _mm256_permutevar8x32_epi32(r_pixel, pixel_move_mask);
                    g_pixel = _mm256_permutevar8x32_epi32(g_pixel, pixel_move_mask);
                    b_pixel = _mm256_permutevar8x32_epi32(b_pixel, pixel_move_mask);

        
                    // [u8 * 16] -> [u16 * 16]に拡張
                    r_pixel = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(r_pixel));
                    g_pixel = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(g_pixel));
                    b_pixel = _mm256_cvtepu8_epi16(_mm256_castsi256_si128(b_pixel));
        
                    // 右シフトで減色
                    r_pixel = _mm256_srli_epi16(r_pixel, 3);
                    g_pixel = _mm256_srli_epi16(g_pixel, 2);
                    b_pixel = _mm256_srli_epi16(b_pixel, 3);
        
                    // 左シフトで合成位置に移動
                    r_pixel = _mm256_slli_epi16(r_pixel, 11);
                    g_pixel = _mm256_slli_epi16(g_pixel, 5);
        
                    // ピクセルに合成
                    let pixel = $endian_fn(_mm256_or_si256(r_pixel, _mm256_or_si256(g_pixel, b_pixel)));
        
                    _mm256_storeu_si256(dst_ptr.cast::<__m256i>(), pixel);
                    
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

encode_from_endian!("big", _mm256_be_epi16, encode_from_rgb888_be, encode_from_rgb565_be, encode_from_rgba8888_be);
encode_from_endian!("little", _mm256_le_epi16, encode_from_rgb888_le, encode_from_rgb565_le, encode_from_rgba8888_le);

fn a() {
    // _mm256_cvtepu8_epi16(_mm256_castsi256_si128(a))
    // _mm256_permutevar8x32_epi32(a, b)
}

fn print256(txt: &str, value: __m256i) {
    let value: M256I8 = unsafe { ::core::mem::transmute(value) };
    print!("{}: 0x", txt);
    for v in value.0 {
        print!(" {:02X}", v);
    }
    println!("");
}

#[cfg(test)]
mod tests {
    use crate::encode::logic::scalar;
    use crate::pixel::PIXEL_BYTES;

    use crate::encode::logic::tests::{NUM_PIXELS, RGB888_DATA, RGB565_DATA, RGBA8888_DATA};

    #[test]
    fn encode_rgb888_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut scalar_buf = [0; NUM_PIXELS * PIXEL_BYTES];
        let mut sse41_buf = [0; NUM_PIXELS * PIXEL_BYTES];

        unsafe {
            scalar::encode_from_rgb888_be(&RGB888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb888_be(&RGB888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        // assert_eq!(scalar_buf, sse41_buf);

        unsafe {
            scalar::encode_from_rgb888_le(&RGB888_DATA, &mut scalar_buf, NUM_PIXELS);
            super::encode_from_rgb888_le(&RGB888_DATA, &mut sse41_buf, NUM_PIXELS);
        }

        // assert_eq!(scalar_buf, sse41_buf);
    }

    #[test]
    fn encode_rgb565_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

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
    fn encode_rgba8888_x86_64_avx2() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

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

    // #[test]
    // fn encode_endian_x86_64() {
    //     let mut a_buf = [0; NUM_PIXELS * PIXEL_BYTES];
    //     let mut b_buf = [0; NUM_PIXELS * PIXEL_BYTES];

    //     let data_ptr = (&RGB565_DATA as *const u16).cast::<u8>();
    //     let data = unsafe { ::core::slice::from_raw_parts(data_ptr, NUM_PIXELS * PIXEL_BYTES) };

    //     unsafe {
    //         super::encode_from_rgb888_be(&RGB888_DATA, &mut a_buf, NUM_PIXELS);

    //         super::encode_from_rgb565_be(data, &mut b_buf, NUM_PIXELS);
    //         assert_eq!(a_buf, b_buf);
    //         super::encode_from_rgba8888_be(&RGBA8888_DATA, &mut b_buf, NUM_PIXELS);
    //         assert_eq!(a_buf, b_buf);

            
    //         super::encode_from_rgb888_le(&RGB888_DATA, &mut a_buf, NUM_PIXELS);

    //         super::encode_from_rgb565_le(data, &mut b_buf, NUM_PIXELS);
    //         assert_eq!(a_buf, b_buf);
    //         super::encode_from_rgba8888_le(&RGBA8888_DATA, &mut b_buf, NUM_PIXELS);
    //         assert_eq!(a_buf, b_buf);
    //     }
    // }

    #[test]
    fn time_check() {
        println!("avx2 : {}", is_x86_feature_detected!("avx2"));

        let data = std::fs::read("pixel_image.raw").unwrap();
        let num_pixels = data.len() / 2;
        let mut scalar_buf = vec![0; data.len() * 2];
        let mut simd_buf = vec![0; data.len() * 2];

        let mut now = std::time::Instant::now();

        unsafe { scalar::encode_from_rgb565_be(&data, &mut scalar_buf, num_pixels) };

        println!("scalar time: {:?}", now.elapsed());

        now = std::time::Instant::now();

        unsafe { super::encode_from_rgb565_be(&data, &mut simd_buf, num_pixels) };

        println!("simd time: {:?}", now.elapsed());

        println!("is equal: {}", scalar_buf == simd_buf);
    }
}