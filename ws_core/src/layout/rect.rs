use glam::Vec2;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub top_left: Vec2,
    pub extents: Vec2,
}

impl Rect {
    pub fn contains(&self, point: &Vec2) -> bool {
        fn contains_1d(min: f32, length: f32, p: f32) -> bool {
            p > min && p < min + length
        }

        contains_1d(self.top_left.x, self.extents.x, point.x)
            && contains_1d(self.top_left.y, self.extents.y, point.y)
    }

    pub fn centre(&self) -> Vec2 {
        Vec2 {
            x: self.top_left.x + (self.extents.x * 0.5),
            y: self.top_left.y + (self.extents.y * 0.5),
        }
    }
}