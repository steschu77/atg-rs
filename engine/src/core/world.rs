use crate::core::{
    camera::Camera,
    car::{Car, Geometry},
    component::{Component, Context},
    game_input, gl_font,
    gl_pipeline::{self, GlMaterial},
    gl_renderer::{RenderContext, RenderObject, Rotation, Transform},
    gl_text::create_text_mesh,
    input,
    player::Player,
    terrain::Terrain,
};
use crate::error::Result;
use crate::sys::opengl as gl;
use crate::v2d::{v3::V3, v4::V4};
use std::path::Path;
use std::rc::Rc;

// ----------------------------------------------------------------------------
pub struct World {
    render_context: RenderContext,
    input_context: game_input::InputContext,
    terrain: Terrain,
    player: Player,
    camera: Camera,
    car: Car,
    debug: RenderObject,
    terrain_chunks: Vec<RenderObject>,
    terrain_normal_arrows: Vec<RenderObject>,
    debug_arrows: Vec<RenderObject>,
    _font: gl_font::Font,
    t: std::time::Duration,
}

// ----------------------------------------------------------------------------
impl World {
    pub fn new(gl: Rc<gl::OpenGlFunctions>) -> Result<Self> {
        let font = gl_font::Font::load(&gl, Path::new("assets/fonts/roboto"))?;
        let mut render_context = RenderContext::new(gl)?;

        let font_id = render_context.insert_material(GlMaterial::Texture {
            texture: font.texture,
        });

        let green_id = render_context.insert_material(GlMaterial::Color {
            color: V3::new([0.0, 1.0, 0.0]), // Green arrows
        });
        let red_id = render_context.insert_material(GlMaterial::Color {
            color: V3::new([1.0, 0.0, 0.0]), // Red arrows
        });
        let blue_id = render_context.insert_material(GlMaterial::Color {
            color: V3::new([0.0, 0.0, 1.0]), // Blue arrows
        });

        let terrain = Terrain::default();
        //let terrain = Terrain::from_png(Path::new("assets/terrain/heightmap.png"))?;
        let camera = Camera::new(
            V4::new([0.0, 4.0, -1.0, 1.0]),
            V4::new([0.0, 0.0, 0.0, 1.0]),
        );
        let t = std::time::Duration::ZERO;

        let mesh = create_text_mesh(&font, "Debug Text: Hello, World!")?;
        let mesh_id = render_context.create_msdftex_mesh(&mesh)?;
        let debug = RenderObject {
            name: String::from("debug"),
            transform: Transform {
                position: V4::new([1.0, 0.0, 0.0, 1.0]),
                rotation: Rotation::default(),
                size: V4::new([1.0, 1.0, 1.0, 1.0]),
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
                position: V4::new([0.0, 0.0, 0.0, 1.0]),
                rotation: Rotation::default(),
                size: V4::new([1.0, 1.0, 1.0, 1.0]),
            },
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id,
            material_id: 0,
            ..Default::default()
        });

        let mut terrain_normal_arrows = Vec::new();
        for x in (0..16u8).step_by(2) {
            for z in (0..16u8).step_by(2) {
                let mesh_id = terrain.create_normal_arrow_mesh(
                    &mut render_context,
                    f32::from(x) + 0.5,
                    f32::from(z) + 0.5,
                    1.0,
                )?;
                terrain_normal_arrows.push(RenderObject {
                    name: format!("terrain_normal_arrow_{x}_{z}"),
                    transform: Transform {
                        position: V4::new([0.0, 0.0, 0.0, 1.0]),
                        rotation: Rotation::default(),
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                    },
                    pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
                    mesh_id,
                    material_id: green_id,
                    ..Default::default()
                });
            }
        }

        use crate::core::gl_pipeline_colored::arrow;
        let x0_arrow_verts = arrow(V3::ZERO, V3::X0)?;
        let x1_arrow_verts = arrow(V3::ZERO, V3::X1)?;
        let x2_arrow_verts = arrow(V3::ZERO, V3::X2)?;
        let x0_debug_arrow_mesh_id =
            render_context.create_colored_mesh(&x0_arrow_verts, &[], true)?;
        let x1_debug_arrow_mesh_id =
            render_context.create_colored_mesh(&x1_arrow_verts, &[], true)?;
        let x2_debug_arrow_mesh_id =
            render_context.create_colored_mesh(&x2_arrow_verts, &[], true)?;
        let debug_arrows = vec![
            RenderObject {
                name: String::from("sphere_debug_arrow"),
                transform: Transform {
                    position: V4::new([0.0, 0.0, 0.0, 1.0]),
                    rotation: Rotation::default(),
                    size: V4::new([1.0, 1.0, 1.0, 1.0]),
                },
                pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
                mesh_id: x0_debug_arrow_mesh_id,
                material_id: green_id,
                ..Default::default()
            },
            RenderObject {
                name: String::from("sphere_debug_arrow"),
                transform: Transform {
                    position: V4::new([0.0, 0.0, 0.0, 1.0]),
                    rotation: Rotation::default(),
                    size: V4::new([1.0, 1.0, 1.0, 1.0]),
                },
                pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
                mesh_id: x1_debug_arrow_mesh_id,
                material_id: red_id,
                ..Default::default()
            },
            RenderObject {
                name: String::from("sphere_debug_arrow"),
                transform: Transform {
                    position: V4::new([0.0, 0.0, 0.0, 1.0]),
                    rotation: Rotation::default(),
                    size: V4::new([1.0, 1.0, 1.0, 1.0]),
                },
                pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
                mesh_id: x2_debug_arrow_mesh_id,
                material_id: blue_id,
                ..Default::default()
            },
        ];

        let player = Player::new(&mut render_context)?;

        let car_geo = Geometry {
            length: 4.0,
            width: 1.7,
            height: 1.5,
            wheel_base: 2.5,
            wheel_track: 2.0,
            wheel_radius: 0.4,
            wheel_width: 0.3,
        };
        let car = Car::new(&mut render_context, car_geo)?;

        Ok(World {
            render_context,
            input_context: game_input::InputContext::default(),
            terrain,
            camera,
            player,
            debug,
            terrain_chunks,
            terrain_normal_arrows,
            debug_arrows,
            car,
            _font: font,
            t,
        })
    }

    pub fn input(&mut self, events: &input::Events, state: input::State) -> Result<()> {
        self.input_context.update_state(state);
        self.camera.input(events)?;
        Ok(())
    }

    pub fn update(&mut self, dt: &std::time::Duration) -> Result<()> {
        self.t += *dt;
        let ctx = Context {
            dt: *dt,
            state: &self.input_context,
            terrain: &self.terrain,
        };

        self.camera.update(&ctx)?;
        self.player.update(&ctx)?;
        self.car.update(&ctx)?;

        self.player.update_debug_arrows(&mut self.render_context)?;

        //let (forward, position) = self.player.transform();
        let (forward, position) = self.car.transform();
        //let (forward, position) = (V4::X2, V4::ZERO);

        let mesh = create_text_mesh(&self._font, &format!("{position}"))?;
        self.render_context
            .update_msdftex_mesh(self.debug.mesh_id, &mesh)?;
        self.debug.transform.position = self.player.position();

        self.camera.look_at(position, forward);
        Ok(())
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn objects(&self) -> Vec<RenderObject> {
        let mut objects = self.terrain_chunks.clone();
        objects.extend(self.terrain_normal_arrows.iter().cloned());
        objects.extend(self.player.objects.iter().cloned());
        objects.extend(self.player.debug_arrows.iter().cloned());
        objects.push(self.debug.clone());
        objects.extend(self.car.objects.iter().cloned());
        objects.extend(self.debug_arrows.iter().cloned());

        objects
    }

    pub fn render_context(&self) -> &RenderContext {
        &self.render_context
    }
}
