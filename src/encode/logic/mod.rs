mod scalar;

#[cfg(any(test, target_arch = "x86", target_arch = "x86_64"))]
mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86_64::*;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use scalar::*;