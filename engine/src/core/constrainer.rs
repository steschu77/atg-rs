use crate::core::component::Context;
use crate::core::gl_pipeline;
use crate::core::gl_renderer::{RenderContext, RenderObject, Transform};
use crate::error::Result;
use crate::v2d::{v3::V3, v4::V4};
use crate::x2d::{constraint::slider::SliderConstraint, rigid_body::RigidBody};

// ----------------------------------------------------------------------------
/// A constraint connecting two rigid bodies, enforcing a reduction in DOF between them.
#[derive(Debug)]
pub struct Constrainer {
    pub debug_arrow: RenderObject,
    pub constraint: SliderConstraint,
    pub pos: V3,
}

// ----------------------------------------------------------------------------
impl Constrainer {
    pub fn new(context: &mut RenderContext, pt_a: V3, pt_b: V3, dir_b: V3) -> Result<Self> {
        use crate::core::gl_pipeline_colored::arrow;
        let pos = V3::new([1.0, 0.0, 0.0]);
        let forward_3d = V3::new([0.0, 0.0, 1.0]);
        let arrow_verts = arrow(pos, pos + 1.5 * forward_3d)?;
        let debug_arrow_mesh_id = context
            .create_colored_mesh(&arrow_verts, &[], true)
            .unwrap();

        let constraint = SliderConstraint::new(pt_a, pt_b, dir_b, 0.2);

        let debug_arrow = RenderObject {
            name: String::from("debug_arrow"),
            transform: Transform {
                position: V4::new([0.0, 0.0, 0.0, 1.0]),
                size: V4::new([1.0, 1.0, 1.0, 1.0]),
                ..Default::default()
            },
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id: debug_arrow_mesh_id,
            material_id: 0,
            ..Default::default()
        };

        Ok(Self {
            debug_arrow,
            constraint,
            pos: V3::zero(),
        })
    }

    // Get the current position of the sphere
    pub fn position(&self) -> V4 {
        V4::from_v3(self.pos, 1.0)
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        use crate::core::gl_pipeline_colored::arrow;
        let arrow_verts = arrow(
            self.constraint.world_anchor_a,
            self.constraint.world_anchor_b,
        )?;
        context.update_colored_mesh(self.debug_arrow.mesh_id, &arrow_verts, &[])?;

        Ok(())
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        body_a: &mut RigidBody,
        body_b: &mut RigidBody,
    ) -> Result<()> {
        let dt = ctx.dt_secs();
        self.constraint.pre_step(body_a, body_b, dt);
        //self.constraint.warm_start(body_a, body_b);
        Ok(())
    }

    pub fn solve(&mut self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        self.constraint.solve(body_a, body_b);
    }
}
