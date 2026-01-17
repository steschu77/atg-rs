use crate::core::gl_graphics;
use crate::error::{Error, Result};
use crate::gfx::color_conversion::{ImageGeometry, ycbcr420_to_rgb24};
use crate::gfx::color_format::ColorFormat;
use crate::sys::opengl::{self as gl, GLint, GLuint};
use std::path::Path;

// ------------------------------------------------------------------------
pub fn load_webp(
    gl: &gl::OpenGlFunctions,
    filter: GLint,
    wrap: GLint,
    path: &Path,
) -> Result<GLuint> {
    let contents = std::fs::read(path)?;
    let frame = miniwebp::read_image(&contents)?;

    let tx_width = frame.mb_width * 16;
    let tx_height = frame.mb_height * 16;
    let geo = ImageGeometry {
        cx: tx_width,
        cy: tx_height,
        cf: ColorFormat::YCbCr420,
    };
    let rgb = ycbcr420_to_rgb24(&frame.ybuf, &frame.ubuf, &frame.vbuf, &geo);

    let texture = gl_graphics::create_texture(gl, tx_width, tx_height, 1, &rgb.data, filter, wrap)?;

    log::info!("Loaded {path:?} as texture {texture} ({tx_width}x{tx_height})");
    Ok(texture)
}

// ------------------------------------------------------------------------
pub fn load_png(
    gl: &gl::OpenGlFunctions,
    filter: GLint,
    wrap: GLint,
    path: &Path,
) -> Result<GLuint> {
    let contents = std::fs::read(path)?;
    let (png, _plte, data) = miniz::png_read::png_read(&contents)?;

    if png.color_type != miniz::png_read::PNGColorType::TrueColorAlpha {
        return Err(Error::InvalidColorFormat);
    }

    let tx_width = (png.width + 3) & !3;
    let tx_height = png.height;

    let mut aligned = vec![0u8; tx_width * tx_height * 4];
    for y in 0..png.height {
        let src_offset = y * (png.width * 4 + 1) + 1;
        let dst_offset = y * tx_width * 4;
        aligned[dst_offset..(dst_offset + png.width * 4)]
            .copy_from_slice(&data[src_offset..(src_offset + png.width * 4)]);
    }

    let texture = gl_graphics::create_texture(gl, tx_width, tx_height, 0, &aligned, filter, wrap)?;

    log::info!("Loaded {path:?} as texture {texture} ({tx_width}x{tx_height})");
    Ok(texture)
}
