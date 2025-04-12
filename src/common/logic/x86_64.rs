#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "x86")]
use ::core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use ::core::arch::x86_64::*;

use ::core::mem::transmute;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct M128I(pub __m128i);

impl M128I {
    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn loadu_si128(mem_addr: *const M128I) -> M128I {
        M128I(_mm_loadu_si128(mem_addr.cast::<__m128i>()))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn storeu_si128(self, mem_addr: *mut M128I) {
        _mm_storeu_si128(mem_addr.cast::<__m128i>(), self.0);
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn slli_epi16<const IMM8: i32>(self) -> M128I {
        M128I(_mm_slli_epi16::<IMM8>(self.0))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn srli_epi16<const IMM8: i32>(self) -> M128I {
        M128I(_mm_srli_epi16::<IMM8>(self.0))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn and_si128(self, a: M128I) -> M128I {
        M128I(_mm_and_si128(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn or_si128(self, a: M128I) -> M128I {
        M128I(_mm_or_si128(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    pub unsafe fn shuffle_epi8(self, a: M128I) -> M128I {
        M128I(_mm_shuffle_epi8(self.0, a.0))
    }
    
    // ---- 追加関数 ----

    #[inline(always)]
    pub const unsafe fn new_i8(e15: i8, e14: i8, e13: i8, e12: i8, e11: i8, e10: i8, e9: i8, e8: i8, e7: i8, e6: i8, e5: i8, e4: i8, e3: i8, e2: i8, e1: i8, e0: i8) -> M128I {
        M128I(transmute([e15, e14, e13, e12, e11, e10, e9, e8, e7, e6, e5, e4, e3, e2, e1, e0]))
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    pub unsafe fn swap_epi16(self) -> M128I {
        const MASK: M128I = unsafe { M128I::new_i8(1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14) };
        self.shuffle_epi8(MASK)
    }

    #[inline]
    #[target_feature(enable = "ssse3")]
    pub unsafe fn be_epi16(self) -> M128I {
        self.swap_epi16()
    }

    #[inline(always)]
    pub unsafe fn le_epi16(self) -> M128I {
        self
    }
}

impl ::core::fmt::Debug for M128I {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl ::core::ops::BitAnd for M128I {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { self.and_si128(rhs) }
    }
}

impl ::core::ops::BitOr for M128I {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { self.or_si128(rhs) }
    }
}


#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct M256I(pub __m256i);

impl M256I {
    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn loadu_si256(mem_addr: *const M256I) -> M256I {
        M256I(_mm256_loadu_si256(mem_addr.cast::<__m256i>()))
    }

    #[inline]
    #[target_feature(enable = "avx")]
    pub unsafe fn storeu_si256(self, mem_addr: *mut M256I) {
        _mm256_storeu_si256(mem_addr.cast::<__m256i>(), self.0);
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn slli_epi16<const IMM8: i32>(self) -> M256I {
        M256I(_mm256_slli_epi16::<IMM8>(self.0))
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn srli_epi16<const IMM8: i32>(self) -> M256I {
        M256I(_mm256_srli_epi16::<IMM8>(self.0))
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn and_si256(self, a: M256I) -> M256I {
        M256I(_mm256_and_si256(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn or_si256(self, a: M256I) -> M256I {
        M256I(_mm256_or_si256(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn shuffle_epi8(self, a: M256I) -> M256I {
        M256I(_mm256_shuffle_epi8(self.0, a.0))
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn permute4x64_epi64<const IMM8: i32>(self) -> M256I {
        M256I(_mm256_permute4x64_epi64::<IMM8>(self.0))
    }

    // ---- 追加関数 ----

    #[inline(always)]
    pub const unsafe fn const_i8<
        const E00: i8, const E01: i8, const E02: i8, const E03: i8, const E04: i8, const E05: i8, const E06: i8, const E07: i8,
        const E08: i8, const E09: i8, const E10: i8, const E11: i8, const E12: i8, const E13: i8, const E14: i8, const E15: i8,
        const E16: i8, const E17: i8, const E18: i8, const E19: i8, const E20: i8, const E21: i8, const E22: i8, const E23: i8,
        const E24: i8, const E25: i8, const E26: i8, const E27: i8, const E28: i8, const E29: i8, const E30: i8, const E31: i8
        >() -> M256I {
        M256I(transmute([
            E00, E01, E02, E03, E04, E05, E06, E07,
            E08, E09, E10, E11, E12, E13, E14, E15,
            E16, E17, E18, E19, E20, E21, E22, E23,
            E24, E25, E26, E27, E28, E29, E30, E31
        ]))
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn swap_epi16(self) -> M256I {
        const MASK: M256I = unsafe { M256I::const_i8::<
            1, 0,  3,  2,  5,  4,  7,  6,
            9, 8, 11, 10, 13, 12, 15, 14,
            1, 0,  3,  2,  5,  4,  7,  6,
            9, 8, 11, 10, 13, 12, 15, 14,
        >()};
        self.shuffle_epi8(MASK)
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn be_epi16(self) -> M256I {
        self.swap_epi16()
    }

    #[inline(always)]
    pub unsafe fn le_epi16(self) -> M256I {
        self   
    }
}

impl ::core::fmt::Debug for M256I {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl ::core::ops::BitAnd for M256I {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { self.and_si256(rhs)}
    }
}

impl ::core::ops::BitOr for M256I {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { self.or_si256(rhs) }
    }
}

