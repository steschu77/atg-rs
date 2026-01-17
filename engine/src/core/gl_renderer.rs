use crate::core::gl_graphics::{
    create_framebuffer, create_program, create_texture_vao, print_opengl_info,
};
use crate::core::world::World;
use crate::core::{IRenderer, gl_pipeline};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::affine4x4;
use crate::v2d::{m4x4::M4x4, v3::V3};
use std::rc::Rc;

const VS_TEXTURE: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPosition;
layout (location = 1) in vec2 aTexCoord;
out vec2 TexCoord;
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

pub struct Renderer {
    gl: Rc<gl::OpenGlFunctions>,
    texture_vao: gl::GLuint,
    texture_program: gl::GLuint,
    fbo: gl::GLuint,
    color_tex: gl::GLuint,
    depth_tex: gl::GLuint,
}

impl Renderer {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        print_opengl_info(&gl);

        let texture_vao = create_texture_vao(&gl);
        let texture_program = create_program(&gl, "texture", VS_TEXTURE, FS_TEXTURE).unwrap();
        let (fbo, color_tex, depth_tex) = create_framebuffer(&gl, 800, 600)?;

        Ok(Self {
            gl,
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

        let mut uniforms = gl_pipeline::GlUniforms {
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

        let objects = world.objects();
        let meshes = world.meshes();
        let materials = world.materials();
        let pipes = world.pipes();

        for object in objects {
            let mesh = meshes.get(object.mesh_id);
            let pipe = pipes.get(object.pipe_id);
            let material = materials.get(object.material_id);
            if let (Some(mesh), Some(material), Some(pipe)) = (mesh, material, pipe) {
                uniforms.model = object.transform.into();
                uniforms.mat_id = object.material_id as gl::GLint;
                pipe.render(mesh, material, &uniforms)?;
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
            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.depth_tex);
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
