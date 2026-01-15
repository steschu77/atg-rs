use super::v2d::v2::V2;
use super::x2d::manifold::Contact;
use super::x2d::manifold::ContactId;
use super::x2d::polygon::Polygon;

// https://www.codeproject.com/Articles/15573/2D-Polygon-Collision-Detection

// ----------------------------------------------------------------------------
struct ReferenceEdge {
    max_separation: f32,
    index: usize,
    flip: bool,
}

// ----------------------------------------------------------------------------
struct ClipVertex {
    id: ContactId,
    v: V2,
}

// ----------------------------------------------------------------------------
struct IncidenceEdge {
    cv: [Contact; 2],
    num_contacts: usize,
}

// ----------------------------------------------------------------------------
// Max separation is the distance poly2 needs to move in direction of n to fix
// a possible collision.
// Find the edge of poly1 (reference edge) with the deepest point of poly2 that
// lies inside poly1.
fn find_reference_edge(poly0: &Polygon, poly1: &Polygon, flip: bool) -> ReferenceEdge {
    let count0 = poly0.count();
    let count1 = poly1.count();

    assert!(count0 <= 8);
    assert!(count1 <= 8);
    let di = [0.0; 8];
    let dij = [0.0; 8];

    let verts0 = poly0.verts();
    let norms0 = poly0.norms();
    let verts1 = poly1.verts();

    for i in 0..count0 {
        let n = norms0[i];
        let v0 = verts0[i];

        // find deepest point j for edge i.
        for j in 0..count1 {
            let v1 = verts1[j];
            dij[j] = n * (v1 - v0);
        }

        // negative values mean "inside" poly1
        di[i] = *dij.iter().min().unwrap();
    }

    // find the maximum negative value, if any
    let (index, max_separation) = di
        .into_iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap();

    ReferenceEdge {
        max_separation,
        index,
        flip,
    }
}

// ----------------------------------------------------------------------------
fn clip_segment(cv: &mut [ClipVertex; 2], d0: f32, d1: f32, clip_edge: u8, idx: usize) {
    let t = d0 / (d0 - d1);
    cv[idx].v = cv[0].v + t * (cv[1].v - cv[0].v);
    cv[idx].id.id[idx] = clip_edge;
    cv[idx].id.id[idx + 2] = 0;
}

// ----------------------------------------------------------------------------
fn clip_segment_to_line(cv: &mut [ClipVertex; 2], normal: &V2, vx: &V2, clip_edge: u8) {
    // Calculate the distance of end points to the line
    let distance0 = normal * (cv[0].v - vx);
    let distance1 = normal * (cv[1].v - vx);

    if (distance0 > 0.0f) {
        clipSegment(cv, distance0, distance1, clip_edge, 0);
    } else if (distance1 > 0.0f) {
        clipSegment(cv, distance0, distance1, clip_edge, 1);
    }
}

// ----------------------------------------------------------------------------
fn find_incident_edge(poly0: &Polygon, poly1: &Polygon, edge: &ReferenceEdge) -> IncidenceEdge {
    let count0 = poly0.count();
    let count1 = poly1.count();

    let iv0 = edge.index;
    let iv1 = if iv0 + 1 < count0 { iv0 + 1 } else { 0 };

    let normal = poly0.norms()[iv0];
    let n2s = poly1.norms();

    let mut dots = [0.0; 8];

    // Find the incident edge on poly1.
    for i in 0..count1 {
        dots[i] = normal * n2s[i];
    }

    // Build the clip vertices for the incident edge.
    let i1 = dots
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap()
        .0;
    let i2 = if i1 + 1 < count1 { i1 + 1 } else { 0 };

    let mut cv = [
        ClipVertex {
            id: ContactId {
                id: [0, 0, edge.index, i1],
            },
            v: poly1.verts()[i1],
        },
        ClipVertex {
            id: ContactId {
                id: [0, 0, edge.index, i2],
            },
            v: poly1.verts()[i2],
        },
    ];

    let v1s = poly0.verts();
    let v10 = v1s[iv0];
    let v11 = v1s[iv1];

    let tangent = normal.perpendicular();

    clip_segment_to_line(&mut cv, -tangent, v10, iv0);
    clip_segment_to_line(&mut cv, tangent, v11, iv1);

    // Now incidentEdge contains the clipping points.
    // Due to roundoff, it is possible that clipping removes all points.
    let mut incident_edge = IncidenceEdge {
        cv,
        num_contacts: 0,
    };

    for i in 0..2 {
        let v = cv[i].v;
        let id = cv[i].id;

        let separation = normal * (v - v10);
        if separation > 0 {
            continue;
        }

        let cp = &mut incident_edge.cv[incident_edge.num_contacts];
        incident_edge.num_contacts += 1;

        cp.separation = separation;
        //cp.position = v;

        if edge.flip {
            cp.normal = -normal;
            cp.id = -id;
        } else {
            cp.normal = normal;
            cp.id = id;
        }
    }
    incident_edge
}

// ----------------------------------------------------------------------------
pub fn collide_polygons(poly0: &Polygon, poly1: &Polygon) -> IncidenceEdge {
    let edge_a = find_reference_edge(poly0, poly1, 0);
    if (edge_a.maxSeparation > 0.0f) {
        return 0;
    }

    let edge_b = find_reference_edge(poly1, poly0, 1);
    if (edge_b.maxSeparation > 0.0f) {
        return 0;
    }

    let ref_edge = if edge_b.maxSeparation > edge_a.maxSeparation {
        &edge_b
    } else {
        &edge_a
    };

    find_incident_edge(poly0, poly1, ref_edge)
}
