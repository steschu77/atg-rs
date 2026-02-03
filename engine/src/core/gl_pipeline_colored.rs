use crate::core::gl_graphics;
use crate::core::gl_pipeline::{GlMaterial, GlMesh, GlPipeline, GlUniforms};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::{m3x3::M3x3, v3::V3};
use std::rc::Rc;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: V3,
    pub n: V3,
}

// --------------------------------------------------------------------------------
fn add_unit_cube_quad(verts: &mut Vec<Vertex>, indices: &mut Vec<u32>, u: V3, v: V3) {
    let i = verts.len() as u32;
    let n = V3::cross(&u, &v);

    #[rustfmt::skip]
    verts.extend_from_slice(&[
        Vertex { pos: 0.5 * (n - u - v), n },
        Vertex { pos: 0.5 * (n + u - v), n },
        Vertex { pos: 0.5 * (n + u + v), n },
        Vertex { pos: 0.5 * (n - u + v), n },
    ]);

    indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
}

// --------------------------------------------------------------------------------
pub fn create_unit_cube_mesh() -> (Vec<Vertex>, Vec<u32>) {
    const U_V_N: [(V3, V3); 3] = [
        (V3::new([1.0, 0.0, 0.0]), V3::new([0.0, 1.0, 0.0])),
        (V3::new([0.0, 1.0, 0.0]), V3::new([0.0, 0.0, 1.0])),
        (V3::new([0.0, 0.0, 1.0]), V3::new([1.0, 0.0, 0.0])),
    ];

    let mut verts = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (u, v) in U_V_N {
        add_unit_cube_quad(&mut verts, &mut indices, u, v);
        add_unit_cube_quad(&mut verts, &mut indices, v, u);
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
pub fn cylinder(sides: usize, radius: f32, height: f32) -> (Vec<Vertex>, Vec<u32>) {
    assert!(sides >= 3);

    let h = V3::new([0.0, height * 0.5, 0.0]);
    let d_theta = std::f32::consts::TAU / (sides as f32);

    // helper to create `sides` points on a circle, incl. seam point
    let mut circle = (0..sides)
        .map(|i| {
            let theta = d_theta * (i as f32);
            let (s, c) = theta.sin_cos();
            (c, s)
        })
        .collect::<Vec<_>>();
    circle.push(circle[0]);

    // top and bottom side vertices
    let mut verts = Vec::with_capacity(circle.len() * 4 + 2);
    for (c, s) in &circle {
        let r = V3::new([radius * c, 0.0, radius * s]);
        let n = V3::new([*c, 0.0, *s]);
        verts.push(Vertex { pos: r + h, n });
        verts.push(Vertex { pos: r - h, n });
    }

    // top and bottom cap rim vertices
    let n0 = V3::new([0.0, 1.0, 0.0]);
    let n1 = V3::new([0.0, -1.0, 0.0]);
    for (c, s) in &circle {
        let r = V3::new([radius * c, 0.0, radius * s]);
        verts.push(Vertex { pos: r + h, n: n0 });
        verts.push(Vertex { pos: r - h, n: n1 });
    }

    // top and bottom cap center vertices
    verts.push(Vertex { pos: h, n: n0 });
    verts.push(Vertex { pos: -h, n: n1 });

    // indices for the cylinder sides
    let mut indices = Vec::with_capacity(sides * 6);
    for i in 0..sides {
        let i0 = (i * 2) as u32;
        indices.extend_from_slice(&[i0, i0 + 2, i0 + 1, i0 + 2, i0 + 3, i0 + 1]);
    }

    // indices for the top and bottom caps
    let rim = circle.len() as u32 * 2;
    let center = circle.len() as u32 * 4;
    for i in 0..sides {
        let rim0 = rim + (i as u32) * 2;
        let rim1 = rim0 + 2;
        indices.extend_from_slice(&[center, rim1, rim0, center + 1, rim0 + 1, rim1 + 1]);
    }

    (verts, indices)
}

// ----------------------------------------------------------------------------
pub fn tetrahedron(side: f32, height: f32) -> Vec<Vertex> {
    let h_tri = side * (3.0_f32).sqrt() * 0.5;

    let v0 = V3::new([-side * 0.5, 0.0, -h_tri / 3.0]);
    let v1 = V3::new([side * 0.5, 0.0, -h_tri / 3.0]);
    let v2 = V3::new([0.0, 0.0, 2.0 * h_tri / 3.0]);
    let v3 = V3::new([0.0, height, 0.0]);
    let n_base = face_normal(v0, v2, v1);
    let n0 = face_normal(v0, v1, v3);
    let n1 = face_normal(v1, v2, v3);
    let n2 = face_normal(v2, v0, v3);

    vec![
        Vertex { pos: v0, n: n_base },
        Vertex { pos: v2, n: n_base },
        Vertex { pos: v1, n: n_base },
        Vertex { pos: v0, n: n0 },
        Vertex { pos: v1, n: n0 },
        Vertex { pos: v3, n: n0 },
        Vertex { pos: v1, n: n1 },
        Vertex { pos: v2, n: n1 },
        Vertex { pos: v3, n: n1 },
        Vertex { pos: v2, n: n2 },
        Vertex { pos: v0, n: n2 },
        Vertex { pos: v3, n: n2 },
    ]
}

// ----------------------------------------------------------------------------
// Creates a debug arrow mesh starting at 'origin', pointing in normalized 'dir'
// direction with given 'length'. Uses tetrahedrons for the arrow shaft and head.
pub fn arrow(origin: V3, n: V3, length: f32) -> Vec<Vertex> {
    let mut verts = Vec::new();

    let v0 = origin;
    let v1 = origin + n * length * 0.8;

    let shaft = tetrahedron(length * 0.1, length * 0.8);
    let head = tetrahedron(length * 0.1, length * 0.2);

    let x_axis = if n.x0().abs() > 0.1 {
        V3::new([0.0, 1.0, 0.0])
    } else {
        V3::new([1.0, 0.0, 0.0])
    };
    let z_axis = n.cross(&x_axis).norm();
    let x_axis = z_axis.cross(&n).norm();

    verts.extend(shaft.iter().map(|v| Vertex {
        pos: v0 + n * v.pos.x1() + x_axis * v.pos.x0() + z_axis * v.pos.x2(),
        n: v.n,
    }));

    verts.extend(head.iter().map(|v| Vertex {
        pos: v1 + n * v.pos.x1() + x_axis * v.pos.x0() + z_axis * v.pos.x2(),
        n: v.n,
    }));

    verts
}

// ----------------------------------------------------------------------------
pub fn transform_mesh(verts: &mut [Vertex], translation: V3, transform: M3x3) {
    for v in verts.iter_mut() {
        v.pos = translation + transform * v.pos;
        v.n = transform * v.n;
    }
}

// ----------------------------------------------------------------------------
fn face_normal(v0: V3, v1: V3, v2: V3) -> V3 {
    let u = v1 - v0;
    let v = v2 - v0;
    V3::cross(&u, &v).norm()
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct GlColoredPipeline {
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
impl GlColoredPipeline {
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
        Ok(GlColoredPipeline {
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

    pub fn create_mesh(
        &self,
        vertices: &[Vertex],
        indices: &[u32],
        is_debug: bool,
    ) -> Result<GlMesh> {
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
        let norm_ofs = std::mem::offset_of!(Vertex, n) as gl::GLint;

        unsafe {
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

        Ok(GlMesh {
            vao_vertices,
            vbo_vertices,
            vbo_indices,
            num_indices,
            num_vertices: vertices.len() as gl::GLsizei,
            primitive_type: gl::TRIANGLES,
            has_indices: !indices.is_empty(),
            is_debug,
        })
    }

    pub fn update_mesh(&self, mesh: &GlMesh, vertices: &[Vertex], indices: &[u32]) {
        let gl = &self.gl;
        unsafe {
            gl_graphics::update_buffer(
                gl,
                mesh.vbo_vertices,
                vertices.as_ptr() as *const _,
                std::mem::size_of_val(vertices),
            );
            if mesh.has_indices {
                gl_graphics::update_buffer(
                    gl,
                    mesh.vbo_indices,
                    indices.as_ptr() as *const _,
                    std::mem::size_of_val(indices),
                );
            }
        }
    }

    pub fn create_cube(&self) -> Result<GlMesh> {
        let (verts, indices) = create_unit_cube_mesh();
        self.create_mesh(&verts, &indices, false)
    }

    pub fn create_plane(&self) -> Result<GlMesh> {
        let (verts, indices) = create_plane_mesh();
        self.create_mesh(&verts, &indices, false)
    }
}

// ----------------------------------------------------------------------------
impl GlPipeline for GlColoredPipeline {
    fn render(
        &self,
        bindings: &GlMesh,
        material: &GlMaterial,
        uniforms: &GlUniforms,
    ) -> Result<()> {
        let gl = &self.gl;
        let color = match material {
            GlMaterial::Color { color } => *color,
            _ => V3::new([1.0, 1.0, 1.0]),
        };
        unsafe {
            gl.UseProgram(self.shader);
            gl.BindVertexArray(bindings.vao_vertices);
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
            gl.Uniform3fv(self.uid_object_color, 1, color.as_ptr());

            #[allow(clippy::collapsible_else_if)]
            if bindings.has_indices {
                if !bindings.is_debug {
                    gl.DrawElements(
                        bindings.primitive_type,
                        bindings.num_indices,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                } else {
                    gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                    gl.DrawElements(
                        bindings.primitive_type,
                        bindings.num_indices,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                    gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                }
            } else {
                if !bindings.is_debug {
                    gl.DrawArrays(bindings.primitive_type, 0, bindings.num_vertices);
                } else {
                    gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                    gl.DrawArrays(bindings.primitive_type, 0, bindings.num_vertices);
                    gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                }
            }
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Drop for GlColoredPipeline {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.shader);
        }
    }
}

// ----------------------------------------------------------------------------
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

// ----------------------------------------------------------------------------
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
