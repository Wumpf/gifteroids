#![cfg(feature = "debug_lines")]

use bevy::prelude::*;
use bevy_prototype_debug_lines::*;

use crate::gifteroids::OrientedBox;
use crate::spaceship::SpaceShip;

pub struct DebugLinesPlugin;

impl Plugin for DebugLinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_prototype_debug_lines::DebugLinesPlugin::default())
            .add_system(draw_obb_debug_lines)
            .add_system(draw_spaceship_debug_lines);
    }
}

fn draw_obb_debug_lines(
    mut lines: ResMut<DebugLines>,
    mut query: Query<(&mut Transform, &OrientedBox)>,
) {
    for (transform, obb) in &mut query {
        lines.line_colored(
            transform.translation - Vec3::from((obb.axis0 - obb.axis1, 0.0)),
            transform.translation - Vec3::from((obb.axis0 + obb.axis1, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
        lines.line_colored(
            transform.translation + Vec3::from((obb.axis0 - obb.axis1, 0.0)),
            transform.translation + Vec3::from((obb.axis0 + obb.axis1, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
        lines.line_colored(
            transform.translation - Vec3::from((obb.axis1 - obb.axis0, 0.0)),
            transform.translation - Vec3::from((obb.axis1 + obb.axis0, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
        lines.line_colored(
            transform.translation + Vec3::from((obb.axis1 - obb.axis0, 0.0)),
            transform.translation + Vec3::from((obb.axis1 + obb.axis0, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
    }
}

#[cfg(feature = "debug_lines")]
fn draw_spaceship_debug_lines(
    mut lines: ResMut<DebugLines>,
    query: Query<&Transform, With<SpaceShip>>,
) {
    for transform in &query {
        let (tri_a, tri_b, tri_c) = SpaceShip::bounding_triangle(transform);

        lines.line_colored(tri_a.extend(0.0), tri_b.extend(0.0), 0.0, Color::ORANGE_RED);
        lines.line_colored(tri_b.extend(0.0), tri_c.extend(0.0), 0.0, Color::ORANGE_RED);
        lines.line_colored(tri_c.extend(0.0), tri_a.extend(0.0), 0.0, Color::ORANGE_RED);
    }
}
