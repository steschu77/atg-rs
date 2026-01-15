use crate::v2d;
use crate::v2d::v2::V2;
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Clone, Copy, Default)]
struct ContactId {
    id: [u8; 4],
}

// ----------------------------------------------------------------------------
#[derive(Clone, Copy, Default)]
struct Contact {
    id: ContactId,
    separation: f32,
    mass_normal: f32,
    mass_tangent: f32,

    bias: f32,
    p_n: f32,  // accumulated normal impulse
    p_t: f32,  // accumulated tangent impulse
    p_nb: f32, // accumulated normal impulse for bias

    position: V2,
    normal: V2,
}

// ----------------------------------------------------------------------------
struct ManifoldKey<'a> {
    b0: &'a mut RigidBody,
    b1: &'a mut RigidBody,
}

// ----------------------------------------------------------------------------
struct Manifold {
    contacts: [Contact; 2],
    num_contacts: u8,
    friction: f32,
}

// ----------------------------------------------------------------------------
impl Manifold {
    // ------------------------------------------------------------------------
    fn new(friction: f32) -> Self {
        Self {
            contacts: [Contact::default(), Contact::default()],
            num_contacts: 0,
            friction,
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, key: &mut ManifoldKey, _dt: f32, inv_dt: f32) {
        let k_allowed_penetration = 0.01;
        let k_bias_factor = 0.2;

        let b0 = &mut key.b0;
        let b1 = &mut key.b1;

        for c in self.contacts.iter_mut().take(self.num_contacts as usize) {
            let tangent = c.normal.perpendicular();

            let r0 = c.position - b0.pos();
            let r1 = c.position - b1.pos();

            let rn0 = r0 * c.normal;
            let rn1 = r1 * c.normal;
            let k_normal = b0.inv_mass()
                + b1.inv_mass()
                + (r0 * r0 - rn0 * rn0) * b0.inv_inertia()
                + (r1 * r1 - rn1 * rn1) * b1.inv_inertia();

            let rt0 = r0 * tangent;
            let rt1 = r1 * tangent;
            let k_tangent = b0.inv_mass()
                + b1.inv_mass()
                + (r0 * r0 - rt0 * rt0) * b0.inv_inertia()
                + (r1 * r1 - rt1 * rt1) * b1.inv_inertia();

            c.mass_normal = 1.0 / k_normal;
            c.mass_tangent = 1.0 / k_tangent;
            c.bias = -k_bias_factor * inv_dt * f32::min(0.0, c.separation + k_allowed_penetration);

            let impulse = c.p_n * c.normal + c.p_t * tangent;
            b0.apply_impulse_at(&-impulse, &c.position);
            b1.apply_impulse_at(&impulse, &c.position);
        }
    }
}
