use glam::{Vec2, Vec4};

#[derive(Debug, Clone)]
pub struct StrokePoint {
    pub position: Vec2,
    pub pressure: f32,
}

#[derive(Debug, Clone)]
pub struct PenBrush {
    pub color: Vec4,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub struct Stroke {
    pub points: Vec<StrokePoint>,
}