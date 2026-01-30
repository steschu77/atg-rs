use crate::error::Result;
use crate::sys::opengl as gl;
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
#[derive(Debug)]
pub struct GlMeshes {
    meshes: Vec<Option<GlMesh>>,
    free_ids: Vec<usize>,
}

// ----------------------------------------------------------------------------
impl GlMeshes {
    // ------------------------------------------------------------------------
    pub fn new(initial: &[GlMesh]) -> Self {
        let meshes = initial.iter().cloned().map(Some).collect();
        GlMeshes {
            meshes,
            free_ids: Vec::new(),
        }
    }

    // ------------------------------------------------------------------------
    pub fn insert(&mut self, mesh: GlMesh) -> usize {
        if let Some(id) = self.free_ids.pop() {
            assert!(id < self.meshes.len());
            assert!(self.meshes[id].is_none());
            self.meshes[id] = Some(mesh);
            id
        } else {
            self.meshes.push(Some(mesh));
            self.meshes.len() - 1
        }
    }

    // ------------------------------------------------------------------------
    pub fn remove(&mut self, id: usize) -> Option<GlMesh> {
        let mesh = self.meshes.get_mut(id);
        if let Some(m) = mesh
            && m.is_some()
        {
            self.free_ids.push(id);
            m.take()
        } else {
            None
        }
    }

    // ------------------------------------------------------------------------
    pub fn get(&self, id: usize) -> Option<&GlMesh> {
        self.meshes.get(id).and_then(|m| m.as_ref())
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct GlMaterials {
    materials: Vec<Option<GlMaterial>>,
    free_ids: Vec<usize>,
}

// ----------------------------------------------------------------------------
impl GlMaterials {
    // ------------------------------------------------------------------------
    pub fn new(initial: &[GlMaterial]) -> Self {
        let materials = initial.iter().cloned().map(Some).collect();
        GlMaterials {
            materials,
            free_ids: Vec::new(),
        }
    }

    // ------------------------------------------------------------------------
    pub fn insert(&mut self, material: GlMaterial) -> usize {
        if let Some(id) = self.free_ids.pop() {
            assert!(id < self.materials.len());
            assert!(self.materials[id].is_none());
            self.materials[id] = Some(material);
            id
        } else {
            self.materials.push(Some(material));
            self.materials.len() - 1
        }
    }

    // ------------------------------------------------------------------------
    pub fn remove(&mut self, id: usize) -> Option<GlMaterial> {
        let material = self.materials.get_mut(id);
        if let Some(m) = material
            && m.is_some()
        {
            self.free_ids.push(id);
            m.take()
        } else {
            None
        }
    }

    // ------------------------------------------------------------------------
    pub fn get(&self, id: usize) -> Option<&GlMaterial> {
        self.materials.get(id).and_then(|m| m.as_ref())
    }
}
