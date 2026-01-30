use crate::core::IRenderer;
use crate::core::camera::Camera;
use crate::core::gl_graphics::{
    create_framebuffer, create_program, create_texture_vao, print_opengl_info,
};
use crate::core::gl_pipeline::{self, GlMaterial};
use crate::core::gl_pipeline_colored::{self, GlColoredPipeline};
use crate::core::gl_pipeline_msdftex::{self, GlMSDFTexPipeline};
use crate::error::{Error, Result};
use crate::sys::opengl as gl;
use crate::v2d::{affine4x4, m4x4::M4x4, v3::V3, v4::V4};
use std::rc::Rc;

// ----------------------------------------------------------------------------
pub struct Renderer {
    gl: Rc<gl::OpenGlFunctions>,
    texture_vao: gl::GLuint,
    texture_program: gl::GLuint,
    fbo: gl::GLuint,
    color_tex: gl::GLuint,
    depth_tex: gl::GLuint,
}

// ----------------------------------------------------------------------------
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

    fn render_1st_pass(
        &self,
        camera: &Camera,
        objects: Vec<RenderObject>,
        context: &RenderContext,
    ) -> Result<()> {
        let gl = &self.gl;

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
            light_pos: V3::new([2.0, 5.0, 2.0]),
            view_pos: cam_pos.into(),
            light_color: V3::new([1.0, 0.5, 1.0]),
            object_color: V3::new([0.5, 1.0, 1.0]),
        };

        let meshes = context.meshes();
        let materials = context.materials();
        let pipes = context.pipes();

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

// ----------------------------------------------------------------------------
impl IRenderer for Renderer {
    fn render(
        &self,
        camera: &Camera,
        objects: Vec<RenderObject>,
        context: &RenderContext,
    ) -> Result<()> {
        self.render_1st_pass(camera, objects, context)?;
        self.render_2nd_pass()?;
        Ok(())
    }

    fn resize(&self, cx: i32, cy: i32) {
        println!("Resize to {cx} x {cy}");
        unsafe { self.gl.Viewport(0, 0, cx, cy) };
    }
}

// ----------------------------------------------------------------------------
pub struct RenderContext {
    gl: Rc<gl::OpenGlFunctions>,
    colored_pipe: Rc<GlColoredPipeline>,
    msdftex_pipe: Rc<GlMSDFTexPipeline>,
    meshes: gl_pipeline::GlMeshes,
    materials: gl_pipeline::GlMaterials,
    pipes: Vec<Rc<dyn gl_pipeline::GlPipeline>>,
}

// ----------------------------------------------------------------------------
impl RenderContext {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        let colored_pipe = Rc::new(GlColoredPipeline::new(Rc::clone(&gl))?);
        let msdftex_pipe = Rc::new(GlMSDFTexPipeline::new(Rc::clone(&gl))?);

        let cube = colored_pipe.create_cube()?;
        let plane = colored_pipe.create_plane()?;

        let meshes = gl_pipeline::GlMeshes::new(&[cube, plane]);
        let materials = gl_pipeline::GlMaterials::new(&[]);

        Ok(RenderContext {
            gl,
            colored_pipe: Rc::clone(&colored_pipe),
            msdftex_pipe: Rc::clone(&msdftex_pipe),
            meshes,
            materials,
            pipes: vec![colored_pipe, msdftex_pipe],
        })
    }

    pub fn insert_material(&mut self, material: GlMaterial) -> usize {
        self.materials.insert(material)
    }

    pub fn create_colored_mesh(
        &mut self,
        vertices: &[gl_pipeline_colored::Vertex],
        indices: &[u32],
        is_debug: bool,
    ) -> Result<usize> {
        let mesh = self.colored_pipe.create_mesh(vertices, indices, is_debug)?;
        Ok(self.meshes.insert(mesh))
    }

    pub fn update_colored_mesh(
        &mut self,
        mesh_id: usize,
        vertices: &[gl_pipeline_colored::Vertex],
        indices: &[u32],
    ) -> Result<()> {
        let mesh = self.meshes.get(mesh_id).ok_or(Error::InvalidMeshId)?;
        self.colored_pipe.update_mesh(mesh, vertices, indices);
        Ok(())
    }

    pub fn create_msdftex_mesh(
        &mut self,
        vertices: &[gl_pipeline_msdftex::Vertex],
    ) -> Result<usize> {
        let mesh = self.msdftex_pipe.create_mesh(vertices)?;
        Ok(self.meshes.insert(mesh))
    }

    pub fn update_msdftex_mesh(
        &mut self,
        mesh_id: usize,
        vertices: &[gl_pipeline_msdftex::Vertex],
    ) -> Result<()> {
        let mesh = self.meshes.get(mesh_id).ok_or(Error::InvalidMeshId)?;
        self.msdftex_pipe.update_mesh(mesh, vertices);
        Ok(())
    }

    pub fn delete_mesh(&mut self, mesh_id: usize) -> Result<()> {
        let mesh = self.meshes.remove(mesh_id).ok_or(Error::InvalidMeshId)?;
        gl_pipeline::delete_mesh(&self.gl, &mesh);
        Ok(())
    }

    pub fn create_cube(&mut self, is_debug: bool) -> Result<usize> {
        let (verts, indices) = gl_pipeline_colored::create_unit_cube_mesh();
        let mesh = self.colored_pipe.create_mesh(&verts, &indices, is_debug)?;
        Ok(self.meshes.insert(mesh))
    }

    pub fn create_plane(&mut self, is_debug: bool) -> Result<usize> {
        let (verts, indices) = gl_pipeline_colored::create_plane_mesh();
        let mesh = self.colored_pipe.create_mesh(&verts, &indices, is_debug)?;
        Ok(self.meshes.insert(mesh))
    }

    pub fn pipes(&self) -> &Vec<Rc<dyn gl_pipeline::GlPipeline>> {
        &self.pipes
    }

    pub fn meshes(&self) -> &gl_pipeline::GlMeshes {
        &self.meshes
    }

    pub fn materials(&self) -> &gl_pipeline::GlMaterials {
        &self.materials
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Copy, Clone)]
pub struct Transform {
    pub position: V4,
    pub rotation: V4,
    pub size: V4,
}

// ----------------------------------------------------------------------------
impl From<Transform> for M4x4 {
    fn from(tx: Transform) -> Self {
        affine4x4::translate(&tx.position)
            * affine4x4::rotate_x1(tx.rotation.x1())
            * affine4x4::rotate_x0(tx.rotation.x0())
            * affine4x4::scale(&tx.size)
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Clone)]
pub struct RenderObject {
    pub name: String,
    pub children: Vec<RenderObject>,
    pub transform: Transform,
    pub pipe_id: usize,
    pub mesh_id: usize,
    pub material_id: usize,
}

// ----------------------------------------------------------------------------
const VS_TEXTURE: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPosition;
layout (location = 1) in vec2 aTexCoord;
out vec2 TexCoord;
void main() {
    gl_Position = vec4(aPosition, 0.0, 1.0);
    TexCoord = aTexCoord;
}"#;

// ----------------------------------------------------------------------------
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
