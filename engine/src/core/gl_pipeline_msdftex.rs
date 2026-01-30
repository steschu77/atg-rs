use crate::core::gl_graphics;
use crate::core::gl_pipeline::{GlMaterial, GlMesh, GlPipeline, GlUniforms};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::v2::V2;
use std::rc::Rc;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: V2,
    pub tex: V2,
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct GlMSDFTexPipeline {
    pub gl: Rc<gl::OpenGlFunctions>,
    pub shader: gl::GLuint,
    pub uid_model: gl::GLint,
    pub uid_view: gl::GLint,
}

// ----------------------------------------------------------------------------
impl GlMSDFTexPipeline {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        let shader = gl_graphics::create_program(&gl, "msdftex", VS_MSDFTEX, FS_MSDFTEX);
        if let Err(e) = shader {
            println!("Error creating shader: {e:?}");
            return Err(e);
        };
        let shader = shader.unwrap();
        let uid_model = gl_graphics::get_uniform_location(&gl, shader, "model").unwrap_or(-1);
        let uid_view = gl_graphics::get_uniform_location(&gl, shader, "camera").unwrap_or(-1);
        Ok(GlMSDFTexPipeline {
            gl,
            shader,
            uid_model,
            uid_view,
        })
    }

    pub fn create_mesh(&self, vertices: &[Vertex]) -> Result<GlMesh> {
        let gl = &self.gl;
        let vao_vertices = gl_graphics::create_vertex_array(gl);
        let vbo_vertices = unsafe {
            gl_graphics::create_buffer(
                gl,
                gl::ARRAY_BUFFER,
                vertices.as_ptr() as *const _,
                std::mem::size_of_val(vertices),
            )
        };

        let stride = std::mem::size_of::<Vertex>() as gl::GLint;
        let pos_ofs = std::mem::offset_of!(Vertex, pos) as gl::GLint;
        let tex_ofs = std::mem::offset_of!(Vertex, tex) as gl::GLint;

        // Define how the vertex attributes are laid out in the VBO
        unsafe {
            gl.EnableVertexAttribArray(0); // position
            gl.VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, pos_ofs as *const _);
            gl.EnableVertexAttribArray(1); // texture
            gl.VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, tex_ofs as *const _);
        }

        Ok(GlMesh {
            vao_vertices,
            vbo_vertices,
            vbo_indices: 0,
            num_indices: 0,
            num_vertices: vertices.len() as gl::GLsizei,
            primitive_type: gl::TRIANGLES,
            has_indices: false,
            is_debug: false,
        })
    }

    pub fn update_mesh(&self, mesh: &GlMesh, vertices: &[Vertex]) {
        let gl = &self.gl;
        unsafe {
            gl_graphics::update_buffer(
                gl,
                mesh.vbo_vertices,
                vertices.as_ptr() as *const _,
                std::mem::size_of_val(vertices),
            );
        }
    }
}

// ----------------------------------------------------------------------------
impl GlPipeline for GlMSDFTexPipeline {
    fn render(&self, mesh: &GlMesh, material: &GlMaterial, uniforms: &GlUniforms) -> Result<()> {
        let gl = &self.gl;
        let texture = match material {
            GlMaterial::Texture { texture } => *texture,
            _ => 0,
        };
        unsafe {
            gl.UseProgram(self.shader);
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, texture);
            gl.UniformMatrix4fv(self.uid_model, 1, gl::FALSE, uniforms.model.as_ptr());
            gl.UniformMatrix4fv(self.uid_view, 1, gl::FALSE, uniforms.camera.as_ptr());
            gl.BindVertexArray(mesh.vao_vertices);
            gl.DrawArrays(mesh.primitive_type, 0, mesh.num_vertices);
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Drop for GlMSDFTexPipeline {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.shader);
        }
    }
}

// ----------------------------------------------------------------------------
const VS_MSDFTEX: &str = r#"
#version 330 core
uniform mat4 model;
uniform mat4 camera;

layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec2 a_tex;

out vec2 v_tex;

void main() {
    // gl_Position = camera * model * vec4(a_pos, 0.0, 1.0);
    vec4 world_pos = vec4(model[3][0], model[3][1], model[3][2], model[3][3]);
    vec4 view_pos = camera * world_pos;
    view_pos.xy += a_pos.xy * 1.01;
    gl_Position = view_pos;
    v_tex = a_tex;
}"#;

// ----------------------------------------------------------------------------
const FS_MSDFTEX: &str = r#"
#version 330 core
uniform sampler2D txtre;

in mediump vec2 v_tex;
out mediump vec4 FragColor;

void main() {
    mediump vec4 color = texture(txtre, v_tex.st);
    mediump float sig_dist = color.a * 2.0 - 1.0;
    mediump float alpha = smoothstep(-0.1, 0.1, sig_dist);
    FragColor = vec4(alpha, alpha, alpha, alpha);
}"#;

// ------------------------------------------------------------------------
pub fn add_plane_quad(verts: &mut Vec<Vertex>, uv: V2, u: f32, v: f32, xy: V2, x: f32, y: f32) {
    #[rustfmt::skip]
    verts.extend_from_slice(&[
        Vertex { pos: xy + V2::new([0.0, 0.0]), tex: uv + V2::new([0.0,   v]) },
        Vertex { pos: xy + V2::new([  x, 0.0]), tex: uv + V2::new([  u,   v]) },
        Vertex { pos: xy + V2::new([0.0,   y]), tex: uv + V2::new([0.0, 0.0]) },
        Vertex { pos: xy + V2::new([0.0,   y]), tex: uv + V2::new([0.0, 0.0]) },
        Vertex { pos: xy + V2::new([  x, 0.0]), tex: uv + V2::new([  u,   v]) },
        Vertex { pos: xy + V2::new([  x,   y]), tex: uv + V2::new([  u, 0.0]) },
    ]);
}
