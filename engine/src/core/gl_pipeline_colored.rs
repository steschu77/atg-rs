use crate::core::gl_graphics;
use crate::core::gl_pipeline::{GlBindings, Uniforms};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::v3::V3;
use std::rc::Rc;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: V3,
    pub n: V3,
}

// --------------------------------------------------------------------------------
pub fn add_cube_quad(verts: &mut Vec<Vertex>, indices: &mut Vec<u32>, u: V3, v: V3) {
    let i = verts.len() as u32;
    let n = V3::cross(&u, &v);
    verts.extend_from_slice(&[
        Vertex { pos: n - u - v, n },
        Vertex { pos: n + u - v, n },
        Vertex { pos: n + u + v, n },
        Vertex { pos: n - u + v, n },
    ]);
    indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
}

// --------------------------------------------------------------------------------
pub fn create_cube_mesh() -> (Vec<Vertex>, Vec<u32>) {
    const U_V_N: [(V3, V3); 3] = [
        (V3::new([1.0, 0.0, 0.0]), V3::new([0.0, 1.0, 0.0])),
        (V3::new([0.0, 1.0, 0.0]), V3::new([0.0, 0.0, 1.0])),
        (V3::new([0.0, 0.0, 1.0]), V3::new([1.0, 0.0, 0.0])),
    ];

    let mut verts = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (u, v) in U_V_N {
        add_cube_quad(&mut verts, &mut indices, u, v);
        add_cube_quad(&mut verts, &mut indices, v, u);
    }

    (verts, indices)
}

// --------------------------------------------------------------------------------
pub fn add_plane_quad(verts: &mut Vec<Vertex>, indices: &mut Vec<u32>, u: V3, v: V3) {
    let i = verts.len() as u32;
    let n = V3::cross(&u, &v);
    verts.extend_from_slice(&[
        Vertex { pos: -u - v, n },
        Vertex { pos: u - v, n },
        Vertex { pos: u + v, n },
        Vertex { pos: -u + v, n },
    ]);
    indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
}

// --------------------------------------------------------------------------------
pub fn create_plane_mesh() -> (Vec<Vertex>, Vec<u32>) {
    let mut verts = Vec::with_capacity(4);
    let mut indices = Vec::with_capacity(6);
    let u = V3::new([1.0, 0.0, 0.0]);
    let v = V3::new([0.0, 0.0, 1.0]);
    add_plane_quad(&mut verts, &mut indices, v, u);

    (verts, indices)
}

// ----------------------------------------------------------------------------
pub struct GlPipeline {
    pub gl: Rc<gl::OpenGlFunctions>,
    pub shader: gl::GLuint,
    pub uid_model: gl::GLint,
    pub uid_view: gl::GLint,
    pub uid_projection: gl::GLint,
    pub uid_camera: gl::GLint,
    pub uid_mat_id: gl::GLint,
    pub uid_light_pos: gl::GLint,
    pub uid_view_pos: gl::GLint,
    pub uid_light_color: gl::GLint,
    pub uid_object_color: gl::GLint,
}

// ----------------------------------------------------------------------------
impl GlPipeline {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        let shader = gl_graphics::create_program(&gl, "gl_pos_col", VS_COLOR, FS_COLOR);
        if let Err(e) = shader {
            println!("Error creating shader: {e:?}");
            return Err(e);
        };
        let shader = shader.unwrap();
        let uid_model = gl_graphics::get_uniform_location(&gl, shader, "model").unwrap_or(-1);
        let uid_view = gl_graphics::get_uniform_location(&gl, shader, "view").unwrap_or(-1);
        let uid_projection =
            gl_graphics::get_uniform_location(&gl, shader, "projection").unwrap_or(-1);
        let uid_camera = gl_graphics::get_uniform_location(&gl, shader, "camera").unwrap_or(-1);
        let uid_mat_id = gl_graphics::get_uniform_location(&gl, shader, "mat_id").unwrap_or(-1);
        let uid_light_pos =
            gl_graphics::get_uniform_location(&gl, shader, "lightPos").unwrap_or(-1);
        let uid_view_pos = gl_graphics::get_uniform_location(&gl, shader, "viewPos").unwrap_or(-1);
        let uid_light_color =
            gl_graphics::get_uniform_location(&gl, shader, "lightColor").unwrap_or(-1);
        let uid_object_color =
            gl_graphics::get_uniform_location(&gl, shader, "objectColor").unwrap_or(-1);
        Ok(GlPipeline {
            gl,
            shader,
            uid_model,
            uid_view,
            uid_projection,
            uid_camera,
            uid_mat_id,
            uid_light_pos,
            uid_view_pos,
            uid_light_color,
            uid_object_color,
        })
    }

    pub fn create_bindings(&self, vertices: &[Vertex], indices: &[u32]) -> Result<GlBindings> {
        let gl = &self.gl;
        let vao = gl_graphics::create_vertex_array(gl);
        let _vbo = unsafe {
            gl_graphics::create_buffer(
                gl,
                gl::ARRAY_BUFFER,
                vertices.as_ptr() as *const _,
                std::mem::size_of_val(vertices),
            )
        };

        let stride = std::mem::size_of::<Vertex>() as gl::GLint;
        let pos_ofs = std::mem::offset_of!(Vertex, pos) as gl::GLint;
        let norm_ofs = std::mem::offset_of!(Vertex, n) as gl::GLint;

        unsafe {
            gl.UseProgram(self.shader);
            gl.EnableVertexAttribArray(0); // position
            gl.EnableVertexAttribArray(1); // normal
            gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, pos_ofs as *const _);
            gl.VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, norm_ofs as *const _);
        }

        let (num_indices, vbo_indices) = if !indices.is_empty() {
            let vbo_indices = unsafe {
                gl_graphics::create_buffer(
                    gl,
                    gl::ELEMENT_ARRAY_BUFFER,
                    indices.as_ptr() as *const _,
                    std::mem::size_of_val(indices),
                )
            };
            (indices.len() as gl::GLsizei, vbo_indices)
        } else {
            (0, 0)
        };

        Ok(GlBindings {
            vao_vertices: vec![vao],
            vbo_indices,
            num_indices,
        })
    }

    pub fn render(&self, bindings: &GlBindings, uniforms: &Uniforms) -> Result<()> {
        let gl = &self.gl;
        unsafe {
            gl.UseProgram(self.shader);
            gl.BindVertexArray(bindings.vao_vertices[0]);
            gl.UniformMatrix4fv(self.uid_model, 1, gl::FALSE, uniforms.model.as_ptr());
            gl.UniformMatrix4fv(self.uid_camera, 1, gl::FALSE, uniforms.camera.as_ptr());
            gl.UniformMatrix4fv(self.uid_view, 1, gl::FALSE, uniforms.view.as_ptr());
            gl.UniformMatrix4fv(
                self.uid_projection,
                1,
                gl::FALSE,
                uniforms.projection.as_ptr(),
            );
            gl.Uniform1i(self.uid_mat_id, uniforms.mat_id);
            gl.Uniform3fv(self.uid_light_pos, 1, uniforms.light_pos.as_ptr());
            gl.Uniform3fv(self.uid_view_pos, 1, uniforms.view_pos.as_ptr());
            gl.Uniform3fv(self.uid_light_color, 1, uniforms.light_color.as_ptr());
            gl.Uniform3fv(self.uid_object_color, 1, uniforms.object_color.as_ptr());
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, bindings.vbo_indices);
            gl.DrawElements(
                gl::TRIANGLES,
                bindings.num_indices,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        Ok(())
    }
}

impl Drop for GlPipeline {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.shader);
        }
    }
}

const VS_COLOR: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_norm;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 camera;

out vec3 v_norm;
out vec3 v_pos;

void main() {
    gl_Position = camera * model * vec4(a_pos, 1.0);
    v_norm = (model * vec4(a_norm, 0.0)).xyz;
    v_pos = (model * vec4(a_pos, 1.0)).xyz;
}"#;

const FS_COLOR: &str = r#"
#version 330 core
in vec3 v_norm;
in vec3 v_pos;

uniform vec3 lightPos; 
uniform vec3 viewPos; 
uniform vec3 lightColor;
uniform vec3 objectColor;

out vec4 FragColor;
void main() {
    // ambient
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;

    // diffuse
    vec3 norm = normalize(v_norm);
    vec3 lightDir = normalize(lightPos - v_pos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
    
    // specular
    float specularStrength = 0.5;
    vec3 viewDir = normalize(viewPos - v_pos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
    vec3 specular = specularStrength * spec * lightColor;
        
    vec3 result = (ambient + diffuse + specular) * objectColor;
    FragColor = vec4(result, 1.0);
}"#;
