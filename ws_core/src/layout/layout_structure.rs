use std::fmt::Debug;

use glam::Vec2;

use crate::{LayoutRectangle, LayoutSizing};

pub trait LayoutStructure: Sized + PartialEq + Debug {
    type Context;

    fn pick(point: Vec2, context: &Self::Context, sizing: &LayoutSizing) -> Option<Self> {
        Self::iter_all(context).find(|x| x.rect(context, sizing).contains(point))
    }

    fn rect(&self, context: &Self::Context, sizing: &LayoutSizing) -> LayoutRectangle {
        LayoutRectangle {
            top_left: self.location(context, sizing),
            extents: self.size(context),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self, context: &Self::Context) -> Vec2;

    fn location(&self, context: &Self::Context, sizing: &LayoutSizing) -> Vec2;

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self>;
}

pub trait LayoutStructureWithFont {
    type FontContext;
    fn font_size(&self, context: &Self::FontContext) -> f32;
}

pub trait LayoutStructureWithStaticText: LayoutStructure {
    fn text(&self, context: &Self::Context) -> &'static str;
}
