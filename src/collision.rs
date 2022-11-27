use crate::gifteroids::OrientedBox;
use bevy::prelude::Vec2;

pub fn point_in_obb(obb: &OrientedBox, bb_pos: Vec2, point: Vec2) -> bool {
    let box_lensq0 = obb.axis0.length_squared();
    let box_lensq1 = obb.axis1.length_squared();
    let to_bb = bb_pos - point;
    let dot0 = obb.axis0.dot(to_bb);
    let dot1 = obb.axis1.dot(to_bb);
    dot0 > -box_lensq0 && dot0 < box_lensq0 && dot1 > -box_lensq1 && dot1 < box_lensq1
}

pub fn line_line_test(start0: Vec2, end0: Vec2, start1: Vec2, end1: Vec2) -> bool {
    let dir0 = end0 - start0;
    let dir1 = end1 - start1;
    let denominator = (dir0.x * dir1.y) - (dir0.y * dir1.x);
    let numerator1 = ((start0.y - start1.y) * dir1.x) - ((start0.x - start1.x) * dir1.y);
    let numerator2 = ((start0.y - start1.y) * dir0.x) - ((start0.x - start1.x) * dir0.y);
    if denominator == 0. {
        return numerator1 == 0. && numerator2 == 0.;
    }

    let r = numerator1 / denominator;
    let s = numerator2 / denominator;
    (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&s)
}
