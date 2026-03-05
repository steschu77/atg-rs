use crate::error::Result;
use crate::sys::opengl as gl;
use crate::util::obj_pool::{ObjId, ObjPool};
use crate::v2d::{m4x4::M4x4, v3::V3};

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlPipelineType {
    Colored = 0,
    MSDFTex = 1,
    RGBATex = 2,
}

// ----------------------------------------------------------------------------
impl From<GlPipelineType> for usize {
    fn from(p: GlPipelineType) -> Self {
        match p {
            GlPipelineType::Colored => 0,
            GlPipelineType::MSDFTex => 1,
            GlPipelineType::RGBATex => 2,
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct GlMesh {
    pub vao_vertices: gl::GLuint,
    pub vbo_vertices: gl::GLuint,
    pub vbo_indices: gl::GLuint,
    pub num_indices: gl::GLsizei,
    pub num_vertices: gl::GLsizei,
    pub primitive_type: gl::GLenum,
    pub has_indices: bool,
    pub is_debug: bool,
}

// ----------------------------------------------------------------------------
pub fn delete_mesh(gl: &gl::OpenGlFunctions, mesh: &GlMesh) {
    unsafe {
        if mesh.vbo_indices != 0 {
            gl.DeleteBuffers(1, &mesh.vbo_indices);
        }
        gl.DeleteBuffers(1, &mesh.vbo_vertices);
        gl.DeleteVertexArrays(1, &mesh.vao_vertices);
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum GlMaterial {
    Texture { texture: gl::GLuint },
    Color { color: V3 },
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
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
    fn render(&self, mesh: &GlMesh, material: &GlMaterial, uniforms: &GlUniforms) -> Result<()>;
}

// ----------------------------------------------------------------------------
pub type GlMeshes = ObjPool<GlMesh>;
pub type GlMeshId = ObjId<GlMesh>;
pub type GlMaterials = ObjPool<GlMaterial>;
pub type GlMaterialId = ObjId<GlMaterial>;
