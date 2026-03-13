use crate::core::gl_renderer::{DefaultMaterials, RenderContext, RenderObject, Transform};
use crate::core::{gl_pipeline, gl_pipeline_colored};
use crate::error::Result;
use crate::v2d::{q::Q, v3::V3, v4::V4};
use crate::x2d::Material;
use crate::x2d::{BodyId, mass::Mass, rigid_body::RigidBody};

// ----------------------------------------------------------------------------
/// A physically simulated sphere that bounces and rolls
#[derive(Debug)]
pub struct PhysicsSphere {
    pub object: RenderObject,
    pub debug_arrow: RenderObject,
    body_id: BodyId,
    radius: f32,
}

// ----------------------------------------------------------------------------
impl PhysicsSphere {
    pub fn new_body(position: V3, radius: f32, mat: Material) -> Result<RigidBody> {
        let density = mat.density;
        let mass = Mass::from_sphere(density, radius)?;
        Ok(RigidBody::new(
            String::from("sphere"),
            mass,
            mat,
            position,
            Q::identity(),
        ))
    }

    pub fn new_sphere(context: &mut RenderContext, body_id: BodyId, radius: f32) -> Result<Self> {
        let (verts, indices) = gl_pipeline_colored::icosphere(1.0, 2);
        let mesh_id = context.create_colored_mesh(&verts, &indices, true)?;

        use crate::core::gl_pipeline_colored::arrow;
        let pos = V3::new([1.0, 0.0, 0.0]);
        let forward_3d = V3::new([0.0, 0.0, 1.0]);
        let arrow_verts = arrow(pos, pos + 1.5 * forward_3d)?;
        let debug_arrow_mesh_id = context
            .create_colored_mesh(&arrow_verts, &[], true)
            .unwrap();

        let object = RenderObject {
            name: String::from("physics_sphere"),
            transform: Transform::default(),
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id,
            material_id: context.default_material(DefaultMaterials::Magenta),
            ..Default::default()
        };

        let debug_arrow = RenderObject {
            name: String::from("debug_arrow"),
            transform: Transform::default(),
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id: debug_arrow_mesh_id,
            material_id: context.default_material(DefaultMaterials::Magenta),
            ..Default::default()
        };

        Ok(Self {
            object,
            debug_arrow,
            radius,
            body_id,
        })
    }

    pub fn id(&self) -> BodyId {
        self.body_id
    }

    /// Get the current position of the sphere
    pub fn position(&self) -> V4 {
        self.object.transform.position
    }

    pub fn transform(&mut self) -> &mut Transform {
        &mut self.object.transform
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        use crate::core::gl_pipeline_colored::arrow;

        let center = self.position().into();
        let v = V3::new([0.0, 0.0, -1.0]);
        let arrow_verts = arrow(center, center + v)?;
        context.update_colored_mesh(self.debug_arrow.mesh_id, &arrow_verts, &[])?;

        Ok(())
    }
}
