
mod sse41;
mod avx2;

#[cfg(target_feature = "avx2")]
pub use avx2::*;

#[cfg(all(not(target_feature = "avx2"), target_feature = "sse4.1"))]
pub use sse41::*;

#[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse4.1")))]
pub use crate::encode::scalar::*;