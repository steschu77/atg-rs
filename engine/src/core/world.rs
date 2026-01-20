use crate::core::IComponent;
use crate::core::game_object::{GameObject, Transform};
use crate::core::gl_font;
use crate::core::gl_pipeline::{self, GlMaterial};
use crate::core::gl_pipeline_colored::{self, GlColoredPipeline};
use crate::core::gl_pipeline_msdftex::GlMSDFTexPipeline;
use crate::core::gl_text::create_text_mesh;
use crate::core::{camera::Camera, input, player::Player, terrain::Terrain};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::{v3::V3, v4::V4};
use std::path::Path;
use std::rc::Rc;

// ----------------------------------------------------------------------------
pub struct World {
    terrain: Terrain,
    player: Player,
    camera: Camera,
    debug: GameObject,
    terrain_chunks: Vec<GameObject>,
    font: gl_font::Font,
    colored_pipe: Rc<GlColoredPipeline>,
    msdftex_pipe: Rc<GlMSDFTexPipeline>,
    pipes: Vec<Rc<dyn gl_pipeline::GlPipeline>>,
    meshes: gl_pipeline::GlMeshes,
    materials: gl_pipeline::GlMaterials,
    t: std::time::Duration,
}

impl World {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        let terrain = Terrain::default();
        let player = Player::default();
        let camera = Camera::new(V4::new([0.0, 4.0, 4.0, 1.0]), V4::new([0.0, 0.0, 0.0, 1.0]));
        let t = std::time::Duration::ZERO;

        let font = gl_font::Font::load(&gl, Path::new("assets/fonts/roboto"))?;

        let colored_pipe = Rc::new(GlColoredPipeline::new(Rc::clone(&gl))?);
        let msdftex_pipe = Rc::new(GlMSDFTexPipeline::new(Rc::clone(&gl))?);

        let cube = colored_pipe.create_cube()?;
        let plane = colored_pipe.create_plane()?;

        let mut meshes = gl_pipeline::GlMeshes::new(&[cube, plane]);
        let materials = gl_pipeline::GlMaterials::new(&[
            GlMaterial::Texture {
                texture: font.texture,
            },
            GlMaterial::Color {
                color: V3::new([0.8, 0.2, 0.2]),
            },
            GlMaterial::Color {
                color: V3::new([0.2, 0.8, 0.2]),
            },
        ]);

        let debug = create_text_mesh(&msdftex_pipe, &font, "Debug Text: Hello, World!")?;
        let debug_mesh_id = meshes.insert_mesh(debug);
        let debug = GameObject {
            name: String::from("debug"),
            transform: Transform {
                position: V4::new([1.0, 0.0, 0.0, 1.0]),
                rotation: V4::default(),
            },
            pipe_id: gl_pipeline::GlPipelineType::MSDFTex.into(),
            mesh_id: debug_mesh_id,
            material_id: 0,
            ..Default::default()
        };

        let mut terrain_chunks = Vec::new();
        let chunk = terrain.create_chunk_mesh(&colored_pipe, 0, 0)?;
        let chunk_mesh_id = meshes.insert_mesh(chunk);
        terrain_chunks.push(GameObject {
            name: String::from("terrain_chunk_0_0"),
            transform: Transform {
                position: V4::new([1.0, 0.0, 0.0, 1.0]),
                rotation: V4::default(),
            },
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id: chunk_mesh_id,
            material_id: 0,
            ..Default::default()
        });

        Ok(World {
            terrain,
            camera,
            player,
            debug,
            terrain_chunks,
            font,
            colored_pipe: Rc::clone(&colored_pipe),
            msdftex_pipe: Rc::clone(&msdftex_pipe),
            pipes: vec![colored_pipe, msdftex_pipe],
            meshes,
            materials,
            t,
        })
    }

    pub fn input(&mut self, events: &input::Events) -> Result<()> {
        self.camera.input(events)?;
        Ok(())
    }

    pub fn update(&mut self, dt: &std::time::Duration, state: &input::State) -> Result<()> {
        self.t += *dt;
        self.terrain.update(&V4::default(), &V4::default())?;
        self.camera.update(dt, state)?;
        self.player.update(dt, state)?;

        self.debug.transform.position =
            self.player.game_object.transform.position + V4::new([0.0, 1.0, 0.0, 0.0]);

        let player_forward = V4::new([
            self.player.rotation.x_axis().x0(),
            0.0,
            self.player.rotation.x_axis().x1(),
            0.0,
        ]);

        self.camera
            .look_at(self.player.game_object.transform.position, player_forward);
        Ok(())
    }

    pub fn objects(&self) -> Vec<GameObject> {
        let mut objects = self.terrain_chunks.clone();
        objects.push(self.player.game_object.clone());
        objects.push(self.debug.clone());
        objects
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

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn create_text(&mut self, text: &str) -> Result<gl_pipeline::GlMesh> {
        create_text_mesh(&self.msdftex_pipe, &self.font, text)
    }

    pub fn create_cube(&self) -> Result<gl_pipeline::GlMesh> {
        let (verts, indices) = gl_pipeline_colored::create_cube_mesh();
        self.colored_pipe.create_bindings(&verts, &indices)
    }

    pub fn create_plane(&self) -> Result<gl_pipeline::GlMesh> {
        let (verts, indices) = gl_pipeline_colored::create_plane_mesh();
        self.colored_pipe.create_bindings(&verts, &indices)
    }
}
