use glam::Vec2;
use crate::rect::Rect;

pub trait LayoutStructure: Sized {
    type Context;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self>;
    //

    fn rect(&self, context: &Self::Context) -> Rect {
        Rect {
            top_left: self.location(context),
            extents: self.size(context),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self, context: &Self::Context) -> Vec2;
    fn location(&self, context: &Self::Context) -> Vec2;
}