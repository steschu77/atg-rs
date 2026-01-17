use crate::core::gl_font::{Font, FontGlyph};
use crate::core::gl_pipeline::GlMesh;
use crate::core::gl_pipeline_msdftex::{GlMSDFTexPipeline, Vertex, add_plane_quad};
use crate::error::Result;
use crate::util::utf8::next_code_point;
use crate::v2d::v2::V2;

// ----------------------------------------------------------------------------
pub fn create_text_mesh(pipe: &GlMSDFTexPipeline, font: &Font, text: &str) -> Result<GlMesh> {
    let mut iter = text.as_bytes().iter();
    let mut pos = V2::new([0.0, 0.0]);
    let mut verts = Vec::new();
    while let Some(ch) = next_code_point(&mut iter) {
        if let Some(glyph) = font.glyphs.get(&ch) {
            add_glyph(glyph, &pos, &mut verts);
            pos += V2::new([glyph.advance, 0.0]);
        }
    }

    let mesh = pipe.create_bindings(&verts)?;
    log::info!(
        "Created text mesh for \"{}\" with {} vertices, vao {}",
        text,
        verts.len(),
        mesh.vao_vertices[0]
    );
    Ok(mesh)
}

// ------------------------------------------------------------------------
fn add_glyph(glyph: &FontGlyph, pos: &V2, verts: &mut Vec<Vertex>) {
    let uv_u = glyph.uv[0];
    let uv_v = 1.0 - glyph.uv[3];
    let uv_width = glyph.uv[2] - glyph.uv[0];
    let uv_height = glyph.uv[3] - glyph.uv[1];
    let uv_pos = V2::new([uv_u, uv_v]);
    let uv_size = V2::new([uv_width, uv_height]);

    let xy_x = glyph.xy[0];
    let xy_y = glyph.xy[1];
    let xy_width = glyph.xy[2] - glyph.xy[0];
    let xy_height = glyph.xy[3] - glyph.xy[1];
    let xy = *pos + V2::new([xy_x, xy_y]);
    let xy_size = V2::new([xy_width, xy_height]);

    add_plane_quad(
        verts,
        uv_pos,
        uv_size.x0(),
        uv_size.x1(),
        xy,
        xy_size.x0(),
        xy_size.x1(),
    );
}
