
#[derive(Debug, Clone, Default,  PartialEq)]
pub struct Insets {
    top: f32,
    left: f32,
    right: f32,
    bottom: f32,
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
