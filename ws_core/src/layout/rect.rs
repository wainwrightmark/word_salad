use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRectangle {
    pub top_left: Vec2,
    pub extents: Vec2,
}

impl LayoutRectangle {
    pub fn contains(&self, point: Vec2) -> bool {
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

    pub fn top_centre(&self)-> Vec2{
        Vec2 {
            x: self.top_left.x + (self.extents.x * 0.5),
            y: self.top_left.y,
        }
    }

    pub fn centre_right(&self) -> Vec2 {
        Vec2 {
            x: self.top_left.x + self.extents.x,
            y: self.top_left.y + (self.extents.y * 0.5),
        }
    }

    pub fn centre_left(&self) -> Vec2 {
        Vec2 {
            x: self.top_left.x,
            y: self.top_left.y + (self.extents.y * 0.5),
        }
    }

    pub fn bottom_left(&self) -> Vec2 {
        Vec2 {
            x: self.top_left.x,
            y: self.top_left.y + self.extents.y,
        }
    }

    pub fn width(&self) -> f32 {
        self.extents.x.abs()
    }

    pub fn height(&self) -> f32 {
        self.extents.y.abs()
    }

    /// If the point in inside this rect, return another point, scaled to 0.0..1.0
    pub fn scaled_inside(&self, point: Vec2) -> Option<Vec2> {
        if !self.contains(point) {
            return None;
        }

        Some((point - self.top_left) / self.extents)
    }
}
