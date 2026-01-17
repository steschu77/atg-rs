use crate::gfx::color_format::ColorFormat;

// ----------------------------------------------------------------------------
pub struct ImageGeometry {
    pub cx: usize,
    pub cy: usize,
    pub cf: ColorFormat,
}

// ----------------------------------------------------------------------------
pub struct Image {
    pub data: Vec<u8>,
    pub stride: usize,
    pub palette: Vec<u32>,
}

// ----------------------------------------------------------------------------
pub fn make_buffersize(stride: usize, cy: usize) -> usize {
    stride * cy
}

// ----------------------------------------------------------------------------
pub fn pal1_to_rgb32(pal1: Image, geo: &ImageGeometry) -> Image {
    let mut rgb32 = Image {
        data: vec![0; geo.cx * geo.cy * 4],
        stride: geo.cx * 4,
        palette: Vec::new(),
    };

    for y in 0..geo.cy {
        let src = &pal1.data[y * pal1.stride..(y + 1) * pal1.stride];
        let dst = &mut rgb32.data[y * rgb32.stride..(y + 1) * rgb32.stride];

        for x in 0..geo.cx {
            let idx = (src[x / 8] >> (7 - (x & 7))) & 1;
            let color = pal1.palette[idx as usize];
            dst[x * 4] = (color >> 16) as u8;
            dst[x * 4 + 1] = (color >> 8) as u8;
            dst[x * 4 + 2] = color as u8;
            dst[x * 4 + 3] = 255;
        }
    }

    rgb32
}

// ----------------------------------------------------------------------------
pub fn pal8_to_rgb32(pal8: Image, geo: &ImageGeometry) -> Image {
    let mut rgb32 = Image {
        data: vec![0; geo.cx * geo.cy * 4],
        stride: geo.cx * 4,
        palette: Vec::new(),
    };

    for y in 0..geo.cy {
        let src = &pal8.data[y * pal8.stride..(y + 1) * pal8.stride];
        let dst = &mut rgb32.data[y * rgb32.stride..(y + 1) * rgb32.stride];

        for x in 0..geo.cx {
            let idx = src[x];
            let color = pal8.palette[idx as usize];
            dst[x * 4] = (color >> 16) as u8;
            dst[x * 4 + 1] = (color >> 8) as u8;
            dst[x * 4 + 2] = color as u8;
            dst[x * 4 + 3] = 255;
        }
    }

    rgb32
}

// ----------------------------------------------------------------------------
pub fn ycbcr420_to_rgb24(ybuf: &[u8], ubuf: &[u8], vbuf: &[u8], geo: &ImageGeometry) -> Image {
    let mut rgb = Image {
        data: vec![0; geo.cx * geo.cy * 3],
        stride: geo.cx * 3,
        palette: Vec::new(),
    };

    for y in 0..geo.cy {
        let ysrc = &ybuf[y * geo.cx..(y + 1) * geo.cx];
        let usrc = &ubuf[(y / 2) * (geo.cx / 2)..(y / 2 + 1) * (geo.cx / 2)];
        let vsrc = &vbuf[(y / 2) * (geo.cx / 2)..(y / 2 + 1) * (geo.cx / 2)];
        let dst = &mut rgb.data[y * rgb.stride..(y + 1) * rgb.stride];

        for x in 0..geo.cx {
            let y = ysrc[x] as f32;
            let u = usrc[x / 2] as f32 - 128.0;
            let v = vsrc[x / 2] as f32 - 128.0;

            let r = (y + 1.402 * v).clamp(0.0, 255.0) as u8;
            let g = (y - 0.344136 * u - 0.714136 * v).clamp(0.0, 255.0) as u8;
            let b = (y + 1.772 * u).clamp(0.0, 255.0) as u8;

            dst[x * 3] = r;
            dst[x * 3 + 1] = g;
            dst[x * 3 + 2] = b;
        }
    }

    rgb
}
