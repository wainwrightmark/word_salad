use std::fmt::Debug;

use glam::Vec2;

use crate::LayoutRectangle;

pub trait LayoutStructure: Sized + PartialEq + Debug {
    type Context;

    fn pick(point: Vec2, context: &Self::Context) -> Option<Self> {
        Self::iter_all(context).find(|x| x.rect(context).contains(point))
    }

    fn rect(&self, context: &Self::Context) -> LayoutRectangle {
        LayoutRectangle {
            top_left: self.location(context),
            extents: self.size(context),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self, context: &Self::Context) -> Vec2;

    fn location(&self, context: &Self::Context) -> Vec2;

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self>;
}

pub trait LayoutStructureWithFont: LayoutStructure {
    fn font_size(&self) -> f32;
}

pub trait LayoutStructureWithStaticText: LayoutStructure {
    fn text(&self, context: &Self::Context) -> &'static str;
}
