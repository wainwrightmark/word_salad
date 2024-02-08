use std::fmt::Debug;

use glam::Vec2;

use crate::{BasicColor, LayoutRectangle, LayoutSizing};

pub trait LayoutStructure: Sized + PartialEq + Debug {
    //TODO rename to positioning
    type Context<'a>;

    fn pick(point: Vec2, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Option<Self> {
        Self::iter_all(context).find(|x| x.rect(context, sizing).contains(point))
    }

    fn rect(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> LayoutRectangle {
        LayoutRectangle {
            top_left: self.location(context, sizing),
            extents: self.size(context, sizing),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2;

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2;

    fn iter_all(context: &Self::Context<'_>) -> impl Iterator<Item = Self>;
}

pub trait LayoutStructureWithFont {
    type FontContext;
    fn font_size(&self, context: &Self::FontContext) -> f32;
}

pub enum TextOrImage{
    Text{
        text: &'static str,
    },
    Image{
        path: &'static str,
        color: BasicColor,
        pressed_color: BasicColor,
        aspect_ratio:f32//width / height,
    }
}

pub trait LayoutStructureWithTextOrImage : LayoutStructure + LayoutStructureWithFont {
    fn text_or_image(&self, context: &Self::Context<'_>) -> TextOrImage;

}
