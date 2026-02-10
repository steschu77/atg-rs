use crate::core::component::{Component, Context};
use crate::core::gl_renderer::{RenderContext, RenderObject, Transform};
use crate::core::{gl_pipeline, gl_pipeline_colored};
use crate::error::Result;
use crate::v2d::{q::Q, v3::V3, v4::V4};
use crate::x2d::{self, mass::Mass, rigid_body::RigidBody};

// ----------------------------------------------------------------------------
/// A physically simulated sphere that bounces and rolls
#[derive(Debug)]
pub struct PhysicsSphere {
    pub object: RenderObject,
    pub debug_arrow: RenderObject,
    body: RigidBody,
    radius: f32,
}

// ----------------------------------------------------------------------------
impl PhysicsSphere {
    pub fn new(context: &mut RenderContext, position: V3, radius: f32) -> Result<Self> {
        let (verts, indices) = gl_pipeline_colored::icosphere(1.0, 2);
        let mesh_id = context.create_colored_mesh(&verts, &indices, true)?;

        use crate::core::gl_pipeline_colored::arrow;
        let pos = V3::new([1.0, 0.0, 0.0]);
        let forward_3d = V3::new([0.0, 0.0, 1.0]);
        let arrow_verts = arrow(pos, pos + 1.5 * forward_3d)?;
        let debug_arrow_mesh_id = context
            .create_colored_mesh(&arrow_verts, &[], true)
            .unwrap();

        let density = x2d::WOOD.density;
        let mass = Mass::from_sphere(density, radius)?;

        let body = RigidBody::new(mass, x2d::WOOD, position, Q::identity());

        let object = RenderObject {
            name: String::from("physics_sphere"),
            transform: Transform {
                position: V4::new([position.x0(), position.x1(), position.x2(), 1.0]),
                size: V4::new([1.0, 1.0, 1.0, 1.0]),
                ..Default::default()
            },
            pipe_id: gl_pipeline::GlPipelineType::Colored.into(),
            mesh_id,
            material_id: 0,
            ..Default::default()
        };

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
            object,
            debug_arrow,
            body,
            radius,
        })
    }

    /// Apply initial linear and angular velocities to the sphere
    ///
    /// # Arguments
    /// * `linear_velocity` - Initial velocity in m/s (e.g., V3::new([2.0, 0.0, 3.0]))
    /// * `angular_velocity` - Initial angular velocity in rad/s (e.g., V3::new([0.0, 5.0, 2.0]))
    pub fn apply_initial_impulse(&mut self, linear_velocity: V3, angular_velocity: V3) {
        // For linear velocity, we need to apply a force that will result in this velocity
        // after one integration step. Since we're applying this before the first update,
        // we can directly set the velocity by applying a large force.
        let dt = 0.016; // Assume ~60 FPS
        let force = linear_velocity * self.body.mass() / dt;
        self.body.apply_force(force);

        // For angular velocity, apply a torque
        // We apply force at an offset to create the desired rotation
        let inertia = self.body.inertia();
        let torque = V3::new([
            angular_velocity.x0() * inertia.x0(),
            angular_velocity.x1() * inertia.x1(),
            angular_velocity.x2() * inertia.x2(),
        ]) / dt;

        // Apply force at radius to create torque
        let offset = V3::new([self.radius, 0.0, 0.0]);
        let force_direction = torque.cross(offset).norm();
        let force_magnitude = torque.length() / self.radius;
        self.body.apply_force_at(
            force_direction * force_magnitude,
            self.body.position() + offset,
        );
    }

    /// Get the current position of the sphere
    pub fn position(&self) -> V4 {
        let pos = self.body.position();
        V4::new([pos.x0(), pos.x1(), pos.x2(), 1.0])
    }

    /// Get the current velocity of the sphere
    pub fn velocity(&self) -> V3 {
        self.body.velocity()
    }

    pub fn transform(&self) -> (V4, V4) {
        let forward = V3::new([0.0, 0.0, -1.0]);
        (V4::from_v3(forward, 1.0), self.position())
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        use crate::core::gl_pipeline_colored::arrow;

        let center = self.body.position() + V3::new([0.0, -self.radius, 0.0]);
        let surface_vel = self.body.velocity_at(center);

        let surface_speed = surface_vel.length();
        if surface_speed > 0.0001 {
            let surface_vel = surface_vel / surface_speed;
            let arrow_verts = arrow(center, center + surface_speed * 0.5 * surface_vel)?;
            context.update_colored_mesh(self.debug_arrow.mesh_id, &arrow_verts, &[])?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Component for PhysicsSphere {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        let dt = ctx.dt_secs();

        self.body.angular_vel = self.body.angular_vel.with_x1(0.0);

        // === Apply Forces ===

        // Gravity
        let gravity_force = V3::new([0.0, -9.81, 0.0]) * self.body.mass();
        self.body.apply_force(gravity_force);

        // Air resistance (simple linear drag)
        let drag_coefficient = 0.1;
        let velocity = self.body.velocity();
        let drag_force = velocity * -drag_coefficient;
        self.body.apply_force(drag_force);

        // === Physics Integration ===
        self.body.integrate(dt);

        // 3. Ground collision detection
        let pos = self.body.position();
        let penetration = 0.0 - (pos.x1() - self.radius);

        if penetration > 0.0 {
            // --- positional correction ---
            let corrected_pos = pos.with_x1(pos.x1() + penetration);
            self.body.pos = corrected_pos;

            let normal = V3::X1;
            let contact = self.body.position() - normal * self.radius;
            let v_contact = self.body.velocity_at(contact);

            let v_n = v_contact.dot(normal);
            //println!("Angular Velocity: {}", self.body.angular_velocity());

            // Only resolve if moving INTO the ground
            if v_n < 0.0 {
                // Coefficient of restitution (bounce)
                let restitution = self.body.restitution();
                let friction = self.body.friction();

                let j_n = -(1.0 + restitution) * v_n * self.body.mass();

                let impulse_n = normal * j_n;
                self.body.apply_impulse_at(impulse_n, contact);

                let v_tangent = v_contact - normal * v_contact.dot(normal);
                let tangent_speed = v_tangent.length();
                //println!("Tangent speed: {}", tangent_speed);

                if tangent_speed > 0.000001 {
                    let tangent = v_tangent / tangent_speed;

                    // Effective mass at contact (linear + angular)
                    let inv_mass = self.body.inv_mass();
                    let inv_inertia = self.body.inv_inertia().x0();

                    let radius2 = self.radius * self.radius;
                    let inv_effective_mass = inv_mass + inv_inertia * radius2;
                    let j_tangent_required = -tangent_speed / inv_effective_mass;
                    let j_tangent_max = friction * j_n.abs();
                    let j_tangent = j_tangent_required.clamp(-j_tangent_max, j_tangent_max);

                    let impulse_tangent = tangent * j_tangent;
                    self.body.apply_impulse_at(impulse_tangent, contact);
                }
                self.body.log();
                //std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }

        // === Update Render Transform ===
        self.object.transform.position = V4::from_v3(self.body.position(), 1.0);
        self.object.transform.rotation = self.body.rotation().into();

        Ok(())
    }
}
