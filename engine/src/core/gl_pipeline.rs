use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::{m4x4::M4x4, v3::V3};

// ----------------------------------------------------------------------------
pub enum GlPipelineType {
    RGBATex = 0,
    MSDFTex = 1,
    Colored = 2,
}

// ----------------------------------------------------------------------------
impl From<GlPipelineType> for usize {
    fn from(p: GlPipelineType) -> Self {
        match p {
            GlPipelineType::RGBATex => 0,
            GlPipelineType::MSDFTex => 1,
            GlPipelineType::Colored => 2,
        }
    }
}

// ----------------------------------------------------------------------------
pub struct GlBindings {
    pub vao_vertices: Vec<gl::GLuint>,
    pub vbo_indices: gl::GLuint,
    pub num_indices: gl::GLsizei,
    pub num_vertices: gl::GLsizei,
}

// ----------------------------------------------------------------------------
pub struct GlUniforms {
    pub model: M4x4,
    pub view: M4x4,
    pub projection: M4x4,
    pub camera: M4x4,
    pub mat_id: gl::GLint,
    pub light_pos: V3,
    pub view_pos: V3,
    pub light_color: V3,
    pub object_color: V3,
}

// --------------------------------------------------------------------------------
pub trait GlPipeline {
    fn render(&self, mesh: &GlBindings, unis: &GlUniforms) -> Result<()>;
}
