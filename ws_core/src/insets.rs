
#[derive(Debug, Clone, Copy, Default,  PartialEq)]
pub struct Insets {
    pub top: f32,
}

impl Insets {
    pub fn new(top: f32, ) -> Self {
        Self {
            top,
        }
    }
}
