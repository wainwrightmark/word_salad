
#[derive(Debug, Clone, Copy, Default,  PartialEq)]
pub struct Insets {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub fn new(top: f32, left: f32, right: f32, bottom: f32) -> Self {
        Self {
            top,
            left,
            right,
            bottom,
        }
    }
}
