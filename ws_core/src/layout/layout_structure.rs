use std::fmt::Debug;

use glam::Vec2;

use crate::LayoutRectangle;

pub trait LayoutStructure: Sized + PartialEq + Debug {
    type Context;
    type Iterator: Iterator<Item = Self>;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self>;

    fn rect(&self, context: &Self::Context) -> LayoutRectangle {
        LayoutRectangle {
            top_left: self.location(context),
            extents: self.size(context),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self, context: &Self::Context) -> Vec2;

    fn location(&self, context: &Self::Context) -> Vec2;

    fn iter_all() -> Self::Iterator;
}

pub trait LayoutStructureWithText {
    fn font_size() -> f32;
}
