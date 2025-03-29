use ::core::fmt;

pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ZeroImageDimensions,
    InputBufferTooSmall,
    OutputBufferTooSmall,
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