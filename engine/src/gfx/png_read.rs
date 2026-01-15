use crate::error::{Error, Result};
use crate::gfx::color_conversion::{
    ImageGeometry, ImagePal, ImageRgb32, pal1_to_rgb32, pal8_to_rgb32,
};
use crate::gfx::color_format::ColorFormat;
use crate::util::inflate::inflate;
use std::mem;

// ----------------------------------------------------------------------------
macro_rules! fourcc {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) << 0) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)
    };
}

// ----------------------------------------------------------------------------
const IHDR: u32 = fourcc!('I', 'H', 'D', 'R');
const IDAT: u32 = fourcc!('I', 'D', 'A', 'T');
const IEND: u32 = fourcc!('I', 'E', 'N', 'D');
const PLTE: u32 = fourcc!('P', 'L', 'T', 'E');

// ----------------------------------------------------------------------------
struct PNGChunkHead {
    length: u32,
    r#type: u32,
}

// ----------------------------------------------------------------------------
struct PNGChunkIHDR {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: PNGColorType,
    compression: u8,
    filter: u8,
    interlace: u8,
    _crc32: u32,
}

// ----------------------------------------------------------------------------
#[repr(u8)]
enum PNGColorType {
    Greyscale = 0,
    TrueColor = 2,
    IndexedColor = 3,
    GreyscaleAplha = 4,
    TrueColorAlpha = 6,
}

// ----------------------------------------------------------------------------
impl From<u8> for PNGColorType {
    fn from(value: u8) -> Self {
        match value {
            0 => PNGColorType::Greyscale,
            2 => PNGColorType::TrueColor,
            3 => PNGColorType::IndexedColor,
            4 => PNGColorType::GreyscaleAplha,
            6 => PNGColorType::TrueColorAlpha,
            _ => panic!("Invalid PNG color type"),
        }
    }
}

// ----------------------------------------------------------------------------
#[repr(u8)]
enum PNGFilterType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

// ----------------------------------------------------------------------------
impl From<u8> for PNGFilterType {
    fn from(value: u8) -> Self {
        match value {
            0 => PNGFilterType::None,
            1 => PNGFilterType::Sub,
            2 => PNGFilterType::Up,
            3 => PNGFilterType::Average,
            4 => PNGFilterType::Paeth,
            _ => panic!("Invalid PNG filter type"),
        }
    }
}

// ----------------------------------------------------------------------------
fn map_color_format(ihdr: &PNGChunkIHDR) -> Option<ColorFormat> {
    match ihdr.color_type {
        PNGColorType::IndexedColor => match ihdr.bit_depth {
            1 => Some(ColorFormat::PAL1),
            2 => Some(ColorFormat::PAL2),
            4 => Some(ColorFormat::PAL4),
            8 => Some(ColorFormat::PAL8),
            _ => None,
        },
        PNGColorType::Greyscale => match ihdr.bit_depth {
            1 => Some(ColorFormat::Y1),
            2 => Some(ColorFormat::Y2),
            4 => Some(ColorFormat::Y4),
            8 => Some(ColorFormat::Y8),
            16 => Some(ColorFormat::Y16),
            _ => None,
        },
        PNGColorType::TrueColor => match ihdr.bit_depth {
            8 => Some(ColorFormat::RGB0888),
            16 => Some(ColorFormat::RGB0ggg),
            _ => None,
        },
        PNGColorType::TrueColorAlpha => match ihdr.bit_depth {
            4 => Some(ColorFormat::RGB4444),
            8 => Some(ColorFormat::RGB8888),
            16 => Some(ColorFormat::RGBgggg),
            _ => None,
        },
        PNGColorType::GreyscaleAplha => match ihdr.bit_depth {
            8 => Some(ColorFormat::YA8),
            16 => Some(ColorFormat::YA16),
            _ => None,
        },
    }
}

// ----------------------------------------------------------------------------
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let pa = u8::abs_diff(b, c) as u32;
    let pb = u8::abs_diff(a, c) as u32;
    let pc = u32::abs_diff(a as u32 + b as u32, 2 * c as u32);

    if pc < pa && pc < pb {
        c
    } else if pb < pa {
        b
    } else {
        a
    }
}

// ----------------------------------------------------------------------------
fn unfilter_scanline0_byte1(recon: &mut [u8], filter_type: PNGFilterType, cx: usize) {
    match filter_type {
        PNGFilterType::None | PNGFilterType::Up => (),
        PNGFilterType::Sub | PNGFilterType::Paeth => {
            // paeth(recon[i-1], 0, 0) is always recon[i-1]
            for i in 1..cx {
                recon[i] += recon[i - 1];
            }
        }
        PNGFilterType::Average => {
            for i in 1..cx {
                recon[i] = recon[i - 1] / 2;
            }
        }
    }
}

// ----------------------------------------------------------------------------
fn unfilter_scanline_n_byte1(
    recon: &mut [u8],
    precon: &[u8],
    filter_type: PNGFilterType,
    cx: usize,
) {
    match filter_type {
        PNGFilterType::None => (),
        PNGFilterType::Sub => {
            for i in 1..cx {
                recon[i] += recon[i - 1];
            }
        }
        PNGFilterType::Up => {
            for i in 0..cx {
                recon[i] += precon[i];
            }
        }
        PNGFilterType::Average => {
            recon[0] += precon[0] / 2;
            for i in 1..cx {
                recon[i] = (recon[i - 1] + precon[i]) / 2;
            }
        }
        PNGFilterType::Paeth => {
            // paeth(0, precon[i], 0) is always precon[i]
            recon[0] += precon[0];

            for i in 1..cx {
                recon[i] += paeth(recon[i - 1], precon[i], precon[i - 1]);
            }
        }
    }
}

// ----------------------------------------------------------------------------
fn unfilter_byte1(data: &mut [u8], stride: usize, geo: &ImageGeometry) {
    let filter_type = data[0].into();
    unfilter_scanline0_byte1(&mut data[1..], filter_type, stride - 1);

    for _ in 1..geo.cy {
        let (prev, data) = data.split_at_mut(stride);
        let filter_type = data[0].into();
        unfilter_scanline_n_byte1(&mut data[1..], &prev[1..], filter_type, stride - 1);
    }
}

// ----------------------------------------------------------------------------
fn decode_idat(idat: &[u8], plte: Vec<u32>, geo: &ImageGeometry) -> Result<ImageRgb32> {
    let size = geo.cy * (geo.cf.stride(geo.cx, 1) + 1);
    let mut data = vec![0u8; size];

    if inflate(&mut data, idat)? != size {
        return Err(Error::InvalidPng);
    }

    match geo.cf {
        ColorFormat::PAL1 => {
            unfilter_byte1(&mut data, geo.cf.stride(geo.cx, 1), geo);
            let pal = ImagePal {
                data: data[1..].to_vec(),
                stride: geo.cf.stride(geo.cx, 1),
                palette: plte,
            };

            return Ok(pal1_to_rgb32(pal, geo));
        }
        ColorFormat::PAL8 => {
            unfilter_byte1(&mut data, geo.cf.stride(geo.cx, 1), geo);
            let pal = ImagePal {
                data: data[1..].to_vec(),
                stride: geo.cf.stride(geo.cx, 1),
                palette: plte,
            };
            return Ok(pal8_to_rgb32(pal, geo));
        }
        _ => {}
    }

    Err(Error::InvalidColorFormat)
}

// ----------------------------------------------------------------------------
pub fn png_read(png: &[u8]) -> Result<ImageRgb32> {
    const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if png.len() < 8 || !png.starts_with(&SIGNATURE) {
        return Err(Error::InvalidPng);
    }

    let mut png = &png[8..png.len()];

    if png.len() < mem::size_of::<PNGChunkHead>() {
        return Err(Error::InvalidPng);
    }

    let head = PNGChunkHead {
        length: u32::from_be_bytes(png[0..4].try_into().unwrap()),
        r#type: u32::from_be_bytes(png[4..8].try_into().unwrap()),
    };

    png = &png[8..png.len()];

    if head.r#type != IHDR {
        return Err(Error::InvalidPng);
    }

    const IHDR_LEN: usize = mem::size_of::<PNGChunkIHDR>();
    if head.length as usize != IHDR_LEN || png.len() < IHDR_LEN {
        return Err(Error::InvalidPng);
    }

    let ihdr = PNGChunkIHDR {
        width: u32::from_be_bytes(png[0..4].try_into().unwrap()),
        height: u32::from_be_bytes(png[4..8].try_into().unwrap()),
        bit_depth: png[8],
        color_type: PNGColorType::from(png[9]),
        compression: png[10],
        filter: png[11],
        interlace: png[12],
        _crc32: u32::from_be_bytes(png[13..17].try_into().unwrap()),
    };

    png = &png[IHDR_LEN..png.len()];

    if ihdr.width == 0
        || ihdr.height == 0
        || ihdr.bit_depth == 0
        || ihdr.compression != 0
        || ihdr.filter != 0
        || ihdr.interlace > 1
    {
        return Err(Error::InvalidPng);
    }

    let geo = ImageGeometry {
        cx: ihdr.width as usize,
        cy: ihdr.height as usize,
        cf: match map_color_format(&ihdr) {
            Some(cf) => cf,
            None => return Err(Error::InvalidPng),
        },
    };

    //let stride = geo.cf.stride(geo.cx, 1);
    //let bufsize = make_buffersize(geo.cf, stride, geo.cy);
    let mut idat = Vec::with_capacity(png.len());
    let mut plte = Vec::new();

    while !png.is_empty() {
        let head = PNGChunkHead {
            length: u32::from_be_bytes(png[0..4].try_into().unwrap()),
            r#type: u32::from_be_bytes(png[4..8].try_into().unwrap()),
        };

        png = &png[8..png.len()];

        match head.r#type {
            IDAT => {
                idat.extend_from_slice(&png[0..head.length as usize]);
            }
            IEND => {
                return decode_idat(&idat, plte, &geo);
            }
            PLTE => {
                if !head.length.is_multiple_of(3) || head.length > 256 * 3 {
                    return Err(Error::InvalidPng);
                }
                for i in (0..head.length as usize).step_by(3) {
                    let r = png[i + 2] as u32;
                    let g = png[i + 1] as u32;
                    let b = png[i] as u32;
                    plte.push((r << 16) | (g << 8) | b);
                }
            }
            _ => {
                // Skip other chunks
            }
        }

        png = &png[head.length as usize..png.len()];
    }

    Err(Error::PngIendMissing)
}
