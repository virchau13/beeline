use crate::{AppState, util::flt_equal};
use bevy::prelude::*;
use impacted::CollisionShape;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Game).with_system(update_collision_transforms),
        );
    }
}

fn update_collision_transforms(
    mut shapes: Query<(&mut CollisionShape, &GlobalTransform), Changed<GlobalTransform>>,
) {
    // Iterate through all collision shapes and set transform accordingly
    for (mut shape, transform) in shapes.iter_mut() {
        shape.set_transform(*transform);
    }
}

/// Parametric line, pos = p + v*t, t scalar, p vector, v vector.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ParaLine {
    pub p: Vec2,
    pub v: Vec2
}

impl ParaLine {
    // Checks if this parametric line intersects another parametric line
    // with scalar values between 0 and 1, and if so, returns the scalar at which it intersects.
    pub fn intersect(&self, other: &Self) -> Option<f32> {
        intersect_lines(self.p, self.v, other.p, other.v)
    }

    // Returns the point at `t`.
    pub fn point(&self, t: f32) -> Vec2 {
        self.p + self.v*t
    }

    pub fn new(p: Vec2, v: Vec2) -> Self {
        Self {
            p,
            v,
        }
    }
}

// Checks if the parametric line p1 + v1*t1 = p2 + v2*t2 for some values 0 <= t1 <= 1, 0 <= t2 <= 1,
// and if so, returns t1.
fn intersect_lines(p1: Vec2, v1: Vec2, p2: Vec2, v2: Vec2) -> Option<f32> {
    let divisor = v1.x*v2.y - v1.y*v2.x;
    if flt_equal(divisor, 0.) {
        // Parallel lines
        return None;
    }
    let t1 = (p2.x*v2.y - p1.x*v2.y + p1.y*v2.x - p2.y*v1.x) / divisor;
    if 0. <= t1 && t1 <= 1. {
        let t2 = if !flt_equal(v2.y, 0.) {
            (v1.y*t1 + p1.y - p2.y) / v2.y
        } else if !flt_equal(v2.x, 0.) {
            (v1.x*t1 + p1.x - p2.x) / v2.x
        } else {
            // t2 can be any value 0 <= t2 <= 1
            0.
        };
        if 0. <= t2 && t2 <= 1. {
            return Some(t1);
        }
    }
    None
}

pub fn rect_to_lines(top_left: Vec2, size: Vec2) -> [ParaLine; 4] {
    let size_x = Vec2::new(size.x, 0.);
    let size_y = Vec2::new(0., size.y); 
    [
        ParaLine::new(top_left, size_x),
        ParaLine::new(top_left, size_y),
        ParaLine::new(top_left + size_x, size_y),
        ParaLine::new(top_left + size_y, size_x)
    ]
}

#[test]
fn intersect_test() {
    assert!(flt_equal(
        intersect_lines(
            Vec2::new(0., -1.),
            Vec2::new(0., 2.),
            Vec2::new(-1., 0.),
            Vec2::new(2., 0.),
        ).unwrap(),
        0.5
    ));
}
