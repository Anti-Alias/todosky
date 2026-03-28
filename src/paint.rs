//! Various reuseable painting functions

use egui::{pos2, Color32, Painter, Pos2, Rect, Stroke};
use crate::TaskGraph;

const EPSILON: f32          = 0.001;
const ARROW_SIDE_LEN: f32   = 10.0;
const CROSS_COLOR: Color32  = Color32::from_rgba_unmultiplied_const(255, 255, 255, 50);
const CROSS_HALF_SIZE: f32  = 200.0;
const CROSS_STROKE: Stroke  = Stroke { width: 1.0, color: CROSS_COLOR };

/// Paints coordinate axis (cross)
pub fn axis(painter: &Painter) {
    let left    = pos2(-CROSS_HALF_SIZE, 0.0);
    let right   = pos2(CROSS_HALF_SIZE, 0.0);
    let top     = pos2(0.0, -CROSS_HALF_SIZE);
    let bottom  = pos2(0.0, CROSS_HALF_SIZE);
    painter.line_segment([left, right], CROSS_STROKE);
    painter.line_segment([top, bottom], CROSS_STROKE);
}

/// Paints line coming out of tasks that have no outgoing connection.
/// This occurs when the user is in the process of wiring dependencies, but hasn't let go of
/// the RMB.
pub fn free_arrows(tasks: &TaskGraph, painter: &Painter, stroke: Stroke) {
    for (_, task) in tasks.iter() {
        let task_center = task.rect().center();
        if let Some(arrow_pos) = task.arrow_pos {
            arrow(task_center, arrow_pos, painter, stroke);
        }
    }
}

/// Paints an arrow between two rectangles in 2d space
pub fn arrow_between_rects(
    source: Rect,
    target: Rect,
    painter: &Painter,
    stroke: Stroke,
) {
    let source_center = source.center();
    let target_center = target.center();
    if source_center.distance_sq(target_center) < EPSILON*EPSILON {     // Don't paint between points that are too close
        return;
    }
    let source_to_target = (target_center - source_center).normalized();
    let target_to_source = -source_to_target;
    let source_point = source.intersects_ray_from_center(source_to_target);
    let target_point = target.intersects_ray_from_center(target_to_source);
    arrow(source_point, target_point, painter, stroke);
}

/// Paints an arrow between two points
pub fn arrow(start: Pos2, end: Pos2, painter: &Painter, stroke: Stroke) {
    if start.distance_sq(end) < EPSILON*EPSILON {
        return; 
    }
    let dir_forwards    = (end - start).normalized() * ARROW_SIDE_LEN;
    let dir_left        = dir_forwards.rot90();
    let tri_back        = end - dir_forwards;
    let tri_left        = tri_back + dir_left;
    let tri_right       = tri_back - dir_left;
    painter.line_segment([start, end], stroke);             // Base of line
    painter.line_segment([tri_left, tri_right], stroke);    // Triangle
    painter.line_segment([tri_right, end], stroke);         // Triangle
    painter.line_segment([end, tri_left], stroke);          // Triangle
}

