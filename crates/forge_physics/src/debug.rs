//! Debug rendering data generation for colliders and contact points.

use forge_core::components::{Collider2D, ColliderShape2D, Transform2D};
use forge_core::Vec2;

/// A debug line segment to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugLine {
    /// Start point in world coordinates.
    pub start: Vec2,
    /// End point in world coordinates.
    pub end: Vec2,
    /// Color `[r, g, b, a]`.
    pub color: [f32; 4],
}

impl DebugLine {
    /// Creates a new debug line.
    pub fn new(start: Vec2, end: Vec2, color: [f32; 4]) -> Self {
        Self { start, end, color }
    }
}

/// Generates wireframe line segments for a given 2D collider transformed into world space.
pub fn generate_collider_debug_lines(
    transform: &Transform2D,
    collider: &Collider2D,
) -> Vec<DebugLine> {
    let mut lines = Vec::new();
    let color = if collider.is_sensor {
        [1.0, 0.8, 0.0, 0.8] // Yellow/orange for sensors
    } else {
        [0.0, 1.0, 0.2, 0.8] // Bright green for solid colliders
    };

    let cos = transform.rotation.cos();
    let sin = transform.rotation.sin();

    // Local offset rotated by transform rotation
    let rot_offset_x = collider.offset.x * cos - collider.offset.y * sin;
    let rot_offset_y = collider.offset.x * sin + collider.offset.y * cos;
    let center = Vec2::new(
        transform.position.x + rot_offset_x,
        transform.position.y + rot_offset_y,
    );

    let rotate_point = |lx: f32, ly: f32| -> Vec2 {
        let rx = lx * cos - ly * sin;
        let ry = lx * sin + ly * cos;
        Vec2::new(center.x + rx, center.y + ry)
    };

    match collider.shape {
        ColliderShape2D::Box { half_extents } => {
            let hx = half_extents.x * transform.scale.x;
            let hy = half_extents.y * transform.scale.y;

            let p0 = rotate_point(-hx, -hy);
            let p1 = rotate_point(hx, -hy);
            let p2 = rotate_point(hx, hy);
            let p3 = rotate_point(-hx, hy);

            lines.push(DebugLine::new(p0, p1, color));
            lines.push(DebugLine::new(p1, p2, color));
            lines.push(DebugLine::new(p2, p3, color));
            lines.push(DebugLine::new(p3, p0, color));
        }
        ColliderShape2D::Circle { radius } => {
            let r = radius * transform.scale.x.abs().max(transform.scale.y.abs());
            let segments = 24;
            let step = std::f32::consts::TAU / segments as f32;

            for i in 0..segments {
                let a1 = i as f32 * step;
                let a2 = (i + 1) as f32 * step;

                let p1 = Vec2::new(center.x + r * a1.cos(), center.y + r * a1.sin());
                let p2 = Vec2::new(center.x + r * a2.cos(), center.y + r * a2.sin());

                lines.push(DebugLine::new(p1, p2, color));
            }
        }
        ColliderShape2D::Capsule {
            radius,
            half_height,
        } => {
            let r = radius * transform.scale.x;
            let hh = half_height * transform.scale.y;

            // Two vertical lines
            let l_top = rotate_point(-r, hh);
            let l_bot = rotate_point(-r, -hh);
            let r_top = rotate_point(r, hh);
            let r_bot = rotate_point(r, -hh);

            lines.push(DebugLine::new(l_top, l_bot, color));
            lines.push(DebugLine::new(r_top, r_bot, color));

            // Top semi-circle and bottom semi-circle
            let half_segs = 12;
            let step = std::f32::consts::PI / half_segs as f32;

            // Top cap
            for i in 0..half_segs {
                let a1 = i as f32 * step;
                let a2 = (i + 1) as f32 * step;

                let p1 = rotate_point(r * a1.cos(), hh + r * a1.sin());
                let p2 = rotate_point(r * a2.cos(), hh + r * a2.sin());
                lines.push(DebugLine::new(p1, p2, color));
            }

            // Bottom cap
            for i in 0..half_segs {
                let a1 = std::f32::consts::PI + i as f32 * step;
                let a2 = std::f32::consts::PI + (i + 1) as f32 * step;

                let p1 = rotate_point(r * a1.cos(), -hh + r * a1.sin());
                let p2 = rotate_point(r * a2.cos(), -hh + r * a2.sin());
                lines.push(DebugLine::new(p1, p2, color));
            }
        }
    }

    lines
}

/// Generates debug cross markers for contact points.
pub fn generate_contact_debug_lines(
    contacts: &[(Vec2, Vec2)], // (point, normal)
) -> Vec<DebugLine> {
    let mut lines = Vec::new();
    let contact_color = [1.0, 0.2, 0.2, 0.9]; // Red for contacts
    let normal_color = [0.2, 0.6, 1.0, 0.9]; // Cyan/blue for normal
    let size = 4.0;

    for &(pt, normal) in contacts {
        // Cross marker
        lines.push(DebugLine::new(
            Vec2::new(pt.x - size, pt.y),
            Vec2::new(pt.x + size, pt.y),
            contact_color,
        ));
        lines.push(DebugLine::new(
            Vec2::new(pt.x, pt.y - size),
            Vec2::new(pt.x, pt.y + size),
            contact_color,
        ));
        // Normal vector
        lines.push(DebugLine::new(
            pt,
            Vec2::new(pt.x + normal.x * 12.0, pt.y + normal.y * 12.0),
            normal_color,
        ));
    }

    lines
}
