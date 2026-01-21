use crate::core::component::{Component, Context};
use crate::core::gl_font;
use crate::core::gl_pipeline::{self, GlMaterial};
use crate::core::gl_renderer::{RenderContext, RenderObject, Transform};
use crate::core::gl_text::create_text_mesh;
use crate::core::{camera::Camera, input, player::Player, terrain::Terrain};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::v4::V4;
use std::path::Path;
use std::rc::Rc;

// ----------------------------------------------------------------------------
pub struct World {
    render_context: RenderContext,
    terrain: Terrain,
    player: Player,
    camera: Camera,
    debug: RenderObject,
    terrain_chunks: Vec<RenderObject>,
    _font: gl_font::Font,
    t: std::time::Duration,
}

impl World {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        let font = gl_font::Font::load(&gl, Path::new("assets/fonts/roboto"))?;
        let mut render_context = RenderContext::new(gl)?;

        let font_id = render_context.insert_material(GlMaterial::Texture {
            texture: font.texture,
        });

        let terrain = Terrain::default();
        let camera = Camera::new(V4::new([0.0, 4.0, 4.0, 1.0]), V4::new([0.0, 0.0, 0.0, 1.0]));
        let t = std::time::Duration::ZERO;

        let mesh_id = create_text_mesh(&mut render_context, &font, "Debug Text: Hello, World!")?;
        let debug = RenderObject {
            name: String::from("debug"),
            transform: Transform {
                position: V4::new([1.0, 0.0, 0.0, 1.0]),
                rotation: V4::default(),
            },
            pipe_id: gl_pipeline::GlPipelineType::MSDFTex.into(),
            mesh_id,
            material_id: font_id,
            ..Default::default()
        };

        let mut terrain_chunks = Vec::new();
        let mesh_id = terrain.create_chunk_mesh(&mut render_context, 0, 0)?;
        terrain_chunks.push(RenderObject {
            name: String::from("terrain_chunk_0_0"),
            transform: Transform {
                position: V4::new([1.0, 0.0, 0.0, 1.0]),
                rotation: V4::default(),
            },
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id,
            material_id: 0,
            ..Default::default()
        });

        let player = Player::new(&mut render_context);

        Ok(World {
            render_context,
            terrain,
            camera,
            player,
            debug,
            terrain_chunks,
            _font: font,
            t,
        })
    }

    pub fn input(&mut self, events: &input::Events) -> Result<()> {
        self.camera.input(events)?;
        Ok(())
    }

    pub fn update(&mut self, dt: &std::time::Duration, state: &input::State) -> Result<()> {
        self.t += *dt;
        let ctx = Context {
            dt: *dt,
            state,
            terrain: &self.terrain,
        };

        self.camera.update(&ctx)?;
        self.player.update(&ctx)?;

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

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn objects(&self) -> Vec<RenderObject> {
        let mut objects = self.terrain_chunks.clone();
        objects.push(self.player.game_object.clone());
        objects.push(self.debug.clone());
        objects
    }

    pub fn render_context(&self) -> &RenderContext {
        &self.render_context
    }
}
