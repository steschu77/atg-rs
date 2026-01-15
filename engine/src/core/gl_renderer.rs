use crate::core::gl_graphics::{
    create_framebuffer, create_program, create_texture_vao, print_opengl_info,
};
use crate::core::gl_pipeline::{GlBindings, v_pos_norm::Vertex};
use crate::core::world::World;
use crate::core::{IRenderer, camera, gl_pipeline};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::affine4x4;
use crate::v2d::{m4x4::M4x4, v3::V3, v4::V4};
use std::rc::Rc;

const VS_TEXTURE: &str = r#"
#version 300 es
layout (location = 0) in vec2 aPosition;
layout (location = 1) in vec2 aTexCoord;
out mediump vec2 TexCoord;
void main() {
    gl_Position = vec4(aPosition, 0.0, 1.0);
    TexCoord = aTexCoord;
}"#;

const FS_TEXTURE: &str = r#"
#version 330 core
in vec2 TexCoord;
out vec4 FragColor;
uniform sampler2D texture1;
float rand(vec2 n) {
    return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}
void main() {
    float n0 = rand( TexCoord.st) - 0.5;
    float n1 = rand(-TexCoord.ts) - 0.5;
    //vec2 noise = 0.05 * vec2(n0*n0, n1*n1);
    vec2 noise = vec2(0.0);
    FragColor = texture(texture1, TexCoord.st + noise);
}"#;

// --------------------------------------------------------------------------------
fn add_cube_quad(verts: &mut Vec<Vertex>, indices: &mut Vec<u32>, u: V3, v: V3) {
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
fn create_cube_mesh() -> (Vec<Vertex>, Vec<u32>) {
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
fn add_plane_quad(verts: &mut Vec<Vertex>, indices: &mut Vec<u32>, u: V3, v: V3) {
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
fn create_plane_mesh() -> (Vec<Vertex>, Vec<u32>) {
    let mut verts = Vec::with_capacity(4);
    let mut indices = Vec::with_capacity(6);
    let u = V3::new([1.0, 0.0, 0.0]);
    let v = V3::new([0.0, 0.0, 1.0]);
    add_plane_quad(&mut verts, &mut indices, v, u);

    (verts, indices)
}

pub struct Renderer {
    gl: Rc<gl::OpenGlFunctions>,
    pipe: gl_pipeline::v_pos_norm::GlPipeline,
    meshes: Vec<GlBindings>,

    texture_vao: gl::GLuint,
    texture_program: gl::GLuint,
    fbo: gl::GLuint,
    color_tex: gl::GLuint,
    depth_tex: gl::GLuint,
}

impl Renderer {
    pub fn new(gl: gl::OpenGlFunctions) -> Result<Self> {
        let gl = Rc::new(gl);
        print_opengl_info(&gl);

        let texture_vao = create_texture_vao(&gl);
        let texture_program = create_program(&gl, "texture", VS_TEXTURE, FS_TEXTURE).unwrap();
        let (fbo, color_tex, depth_tex) = create_framebuffer(&gl, 800, 600)?;

        let pipe = gl_pipeline::v_pos_norm::GlPipeline::new(Rc::clone(&gl))?;
        let (verts, indices) = create_cube_mesh();
        let cube = pipe.create_bindings(&verts, &indices)?;
        let (verts, indices) = create_plane_mesh();
        let plane = pipe.create_bindings(&verts, &indices)?;

        Ok(Self {
            pipe,
            gl,
            meshes: vec![cube, plane],
            texture_vao,
            texture_program,
            fbo,
            color_tex,
            depth_tex,
        })
    }

    fn render_1st_pass(&self, world: &World) -> Result<()> {
        let gl = &self.gl;

        let camera = world.camera();
        let view = camera.transform();
        let cam_pos = camera.position();
        let projection = affine4x4::perspective(45.0, 800.0 / 600.0, 0.1, 100.0);
        let camera = projection * view;

        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::CULL_FACE);
            gl.ClearColor(0.3, 0.2, 0.1, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        /*
                let view = affine4x4::look_at(
                    V4::new([1.0, 3.0, 3.0, 1.0]),
                    V4::new([0.0, 0.0, 0.0, 1.0]),
                    V4::new([0.0, 0.0, 1.0, 0.0]),
                );

                let projection = affine4x4::perspective(45.0, 800.0 / 600.0, 0.1, 100.0);
                let camera = projection * view;
        */
        let mut uniforms = gl_pipeline::v_pos_norm::Uniforms {
            model: M4x4::identity(),
            view,
            projection,
            camera,
            mat_id: 0,
            light_pos: V3::new([2.0, 2.0, 2.0]),
            view_pos: cam_pos.into(),
            light_color: V3::new([1.0, 0.5, 1.0]),
            object_color: V3::new([0.5, 1.0, 1.0]),
        };

        for entity in world.mesh_entities() {
            if let Some(mesh) = self.meshes.get(entity.mesh_id as usize) {
                uniforms.model = entity.transform.into();
                uniforms.mat_id = entity.material_id as gl::GLint;
                self.pipe.render(mesh, &uniforms)?;
            }
        }

        Ok(())
    }

    fn render_2nd_pass(&self) -> Result<()> {
        let gl = &self.gl;
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl.Disable(gl::DEPTH_TEST);

            gl.UseProgram(self.texture_program);
            gl.BindVertexArray(self.texture_vao);
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.color_tex);
            gl.DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
        Ok(())
    }
}

impl IRenderer for Renderer {
    fn render(&self, world: &World) -> Result<()> {
        self.render_1st_pass(world)?;
        self.render_2nd_pass()?;
        Ok(())
    }

    fn resize(&self, cx: i32, cy: i32) {
        println!("Resize to {cx} x {cy}");
        unsafe { self.gl.Viewport(0, 0, cx, cy) };
    }
}
