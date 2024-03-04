use std::fmt::Debug;

use glam::Vec2;
use strum::{Display, EnumIs};

use crate::{BackgroundType, BasicColor, LayoutRectangle, LayoutSizing};

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


pub trait LayoutStructureWithOrigin : LayoutStructure{
    fn origin(&self, context: &Self::Context<'_>, sizing: &LayoutSizing)-> Origin;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, EnumIs)]
pub enum Origin{
    Center,
    TopLeft,
    CenterLeft
}

pub trait LayoutStructureWithFont {
    type FontContext;
    fn font_size(&self, context: &Self::FontContext) -> f32;
}

pub enum TextOrImage {
    Text {
        text: &'static str,
    },
    Image {
        path: &'static str,
        color: BasicColor,
        pressed_color: BasicColor,
        aspect_ratio: f32, //width / height,
    },
}

pub trait LayoutStructureWithTextOrImage: LayoutStructure + LayoutStructureWithFont {
    fn text_or_image(&self, context: &Self::Context<'_>) -> TextOrImage;
}

pub trait LayoutStructureDoubleTextButton:
    LayoutStructure + LayoutStructureWithFont<FontContext = ()>
{
    type TextContext<'a>;

    fn double_text(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String);

    fn left_font(&self) -> &'static str;
    fn right_font(&self) -> &'static str;

    fn text_color(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> BasicColor;
    fn fill_color(
        &self,
        background_type: BackgroundType,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> BasicColor;

    fn is_disabled(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> bool;
}
