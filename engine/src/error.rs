// ----------------------------------------------------------------------------
#[derive(Debug, PartialEq)]
pub enum Error {
    GameOver,
    Underflow,
    Overflow,
    InvalidHeader,
    InvalidBitstream,
    InvalidBlockType,
    InvalidBlockLength,
    InvalidCodeLength,
    InvalidDistance,
    InvalidLength,
    InvalidSymbol,
    InvalidData,
    UnderSubscribedTree,
    OverSubscribedTree,
    InvalidPng,
    PngIendMissing,
    InvalidColorFormat,
    InvalidCString,
    InvalidLocation,
    OpenGLLoadError {
        name: String,
    },
    ShaderCompileError {
        name: String,
        log: String,
    },
    ShaderLinkError {
        name: String,
        log: String,
    },
    GpuOutOfMemory,
    InvalidTextureFormat,
    InvalidTextureSize,
    DeviceNotFound,
    FramebufferIncomplete {
        status: u32,
    },
    OpenGl {
        code: u32,
    },
    FileIo {
        err: std::io::ErrorKind,
    },
    ParseInt {
        err: std::num::ParseIntError,
    },
    Serde {
        line: usize,
        column: usize,
        msg: String,
    },
    WebP {
        err: miniwebp::Error,
    },
    Png {
        err: miniz::png_read::Error,
    },
    Win32Error {
        code: i32,
    },
}

// ----------------------------------------------------------------------------
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let err = format!("{self:?}");
        f.write_str(&err)
    }
}

impl std::error::Error for Error {}

// ----------------------------------------------------------------------------
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileIo { err: err.kind() }
    }
}

// ----------------------------------------------------------------------------
impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseInt { err }
    }
}

// ----------------------------------------------------------------------------
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde {
            line: err.line(),
            column: err.column(),
            msg: err.to_string(),
        }
    }
}

// ----------------------------------------------------------------------------
impl From<miniwebp::Error> for Error {
    fn from(err: miniwebp::Error) -> Self {
        Error::WebP { err }
    }
}

// ----------------------------------------------------------------------------
impl From<miniz::png_read::Error> for Error {
    fn from(err: miniz::png_read::Error) -> Self {
        Error::Png { err }
    }
}

// ----------------------------------------------------------------------------
#[cfg(target_os = "windows")]
impl From<windows::core::Error> for Error {
    fn from(err: windows::core::Error) -> Self {
        Error::Win32Error { code: err.code().0 }
    }
}

// ----------------------------------------------------------------------------
pub type Result<T> = std::result::Result<T, Error>;
