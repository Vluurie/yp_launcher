use glam::Vec3;

pub(super) type Shape = (Vec3, Vec3, f32);

pub(super) fn push(point: Vec3, shapes: &[Shape], strength: f32, fallback: Vec3) -> Vec3 {
    let mut adjusted = point;
    for &(a, b, radius) in shapes {
        let axis = b - a;
        let length = axis.length_squared();
        let t = if length > 0. {
            ((adjusted - a).dot(axis) / length).clamp(0., 1.)
        } else {
            0.
        };
        let offset = adjusted - (a + axis * t);
        let distance = offset.length();
        if distance < radius {
            let normal = offset.try_normalize().unwrap_or(fallback);
            adjusted += normal * ((radius - distance) * strength);
        }
    }
    adjusted - point
}

fn closest_to_triangle(p: Vec3, [a, b, c]: [Vec3; 3]) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0. && d2 <= 0. {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0. && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0. && d1 >= 0. && d3 <= 0. {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0. && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0. && d2 >= 0. && d6 <= 0. {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0. && d4 >= d3 && d5 >= d6 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = (va + vb + vc).recip();
    a + ab * (vb * denom) + ac * (vc * denom)
}

pub(super) fn triangle(mut vertices: [Vec3; 3], spheres: &[(Vec3, f32)], strength: f32) -> Vec3 {
    let Some(normal) = (vertices[1] - vertices[0])
        .cross(vertices[2] - vertices[0])
        .try_normalize()
    else {
        return Vec3::ZERO;
    };
    let mut total = Vec3::ZERO;
    for &(center, radius) in spheres {
        let nearest = closest_to_triangle(center, vertices);
        let depth = radius - center.distance(nearest);
        if depth > 0. {
            let delta = normal * (depth * strength);
            for vertex in &mut vertices {
                *vertex += delta;
            }
            total += delta;
        }
    }
    total
}
