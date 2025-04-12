use ::core::fmt;

pub type Result<T> = ::core::result::Result<T, Error>;

/// Errors that can occur during encoding or decoding.
#[derive(Debug)]
pub enum Error {
    /// The image has a width or height of zero.
    ZeroImageDimensions,

    /// The input buffer does not contain enough data.
    InputBufferTooSmall,

    /// The output buffer is too small to hold the result.
    OutputBufferTooSmall,

    /// The image format or header is not supported or invalid.
    UnsupportedFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ZeroImageDimensions => write!(f, "Image width or height is zero"),
            Error::InputBufferTooSmall => write!(f, "Input buffer is too small"),
            Error::OutputBufferTooSmall => write!(f, "Output buffer is too small"),
            Error::UnsupportedFormat => write!(f, "Unsupported image format or header"),
        }
    }
}