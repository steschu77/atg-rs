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
    OpenGLLoadError { name: String },
    ShaderCompileError { name: String, log: String },
    ShaderLinkError { name: String, log: String },
    GpuOutOfMemory,
    InvalidTextureFormat,
    InvalidTextureSize,
    DeviceNotFound,
    Win32Error { code: i32 },
    OpenGl { code: u32 },
    FramebufferIncomplete { status: u32 },
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
impl From<windows::core::Error> for Error {
    fn from(err: windows::core::Error) -> Self {
        Error::Win32Error { code: err.code().0 }
    }
}

// ----------------------------------------------------------------------------
pub type Result<T> = std::result::Result<T, Error>;
