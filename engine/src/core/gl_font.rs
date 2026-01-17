// Read and use multi-channel signed distance field (MSDF) atlas for font rendering using the MSDFTex
// pipeline in the renderer.
//
// Generate MSDF font atlas using msdf-atlas-gen tool from:
// https://github.com/Chlumsky/msdf-atlas-gen?tab=readme-ov-file
//
// Example command to generate MSDF font atlas:
// msdf-atlas-gen.exe -font Roboto.ttf -type mtsdf -fontname Roboto -format png -imageout roboto.png -json roboto.json -charset charset_all.txt -pots

use crate::core::gl_texture;
use crate::error::Result;
use crate::sys::opengl::{self as gl, GLuint};
use serde::Deserialize;

#[derive(Clone)]
pub struct Font {
    pub width: usize,
    pub height: usize,
    pub texture: GLuint,
    pub meta: FontMeta,
    pub glyphs: FontGlyphs,
}

#[derive(Debug, Clone)]
pub struct FontMeta {
    pub line_height: f32,
    pub _ascender: f32,
    pub _descender: f32,
    pub _underline_y: f32,
    pub _underline_thickness: f32,
}

#[derive(Debug, Clone)]
pub struct FontGlyph {
    pub uv: [f32; 4],
    pub xy: [f32; 4],
    pub advance: f32,
}

type FontGlyphs = std::collections::HashMap<u32, FontGlyph>;

impl FontGlyph {
    fn new(glyph: &JsonGlyph, size: (f32, f32)) -> Self {
        let uv = if let Some(b) = &glyph.atlas_bounds {
            [
                size.0 * b.left,
                size.1 * b.bottom,
                size.0 * b.right,
                size.1 * b.top,
            ]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };
        let xy = if let Some(bounds) = &glyph.plane_bounds {
            [bounds.left, bounds.bottom, bounds.right, bounds.top]
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };
        Self {
            uv,
            xy,
            advance: glyph.advance,
        }
    }
}

impl Font {
    pub fn load(gl: &gl::OpenGlFunctions, path: &std::path::Path) -> Result<Self> {
        let png_path = path.with_extension("png");
        let (width, height, texture) =
            gl_texture::load_png(gl, gl::LINEAR, gl::CLAMP_TO_EDGE, &png_path)?;

        let size = (1.0 / width as f32, 1.0 / height as f32);
        let json_path = path.with_extension("json");
        let (meta, glyphs) = load_json(&json_path, size)?;

        Ok(Self {
            width,
            height,
            texture,
            meta,
            glyphs,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonGlyphAtlas {
    pub metrics: JsonMetrics,
    pub glyphs: Vec<JsonGlyph>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonMetrics {
    pub line_height: f32,
    pub ascender: f32,
    pub descender: f32,
    pub underline_y: f32,
    pub underline_thickness: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonGlyph {
    pub unicode: u32,
    pub advance: f32,
    pub plane_bounds: Option<JsonBounds>,
    pub atlas_bounds: Option<JsonBounds>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonBounds {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

fn load_json(path: &std::path::Path, size: (f32, f32)) -> Result<(FontMeta, FontGlyphs)> {
    let contents = std::fs::read_to_string(path)?;
    let atlas = serde_json::from_str::<JsonGlyphAtlas>(&contents)?;

    let mut glyphs = FontGlyphs::new();
    for glyph in atlas.glyphs.iter() {
        let g = FontGlyph::new(glyph, size);
        glyphs.insert(glyph.unicode, g);
    }

    let meta = FontMeta {
        line_height: atlas.metrics.line_height,
        _ascender: atlas.metrics.ascender,
        _descender: atlas.metrics.descender,
        _underline_y: atlas.metrics.underline_y,
        _underline_thickness: atlas.metrics.underline_thickness,
    };

    Ok((meta, glyphs))
}
