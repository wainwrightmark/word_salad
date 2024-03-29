use nice_bevy_utils::window_size::*;
use ws_core::layout::entities::*;

pub use crate::prelude::*;

pub type Size = WindowSize<SaladWindowBreakPoints>;

#[derive(Debug, Clone, Copy)]
pub struct MyWindowSize; //TODO rename

impl NodeContext for MyWindowSize {
    type Wrapper<'c> = Res<'c, Size>;

    fn has_changed(wrapper: &Self::Wrapper<'_>) -> bool {
        wrapper.is_changed()
    }
}

#[derive(Debug, Default)]
pub struct SaladWindowBreakPoints;

impl Breakpoints for SaladWindowBreakPoints {
    fn size_scale(_: f32, _: f32) -> f32 {
        1.0
    }
}

pub trait SaladWindowSize {
    fn scale(&self) -> f32;

    fn font_size<T: LayoutStructureWithFont>(&self, entity: &T, context: &T::FontContext) -> f32;
    fn tile_size(&self, selfie_mode: &SelfieMode, insets: Insets) -> f32;

    fn get_rect<T: LayoutStructure>(&self, entity: &T, context: &T::Context<'_>)
        -> LayoutRectangle;
    fn try_pick_with_tolerance<T: LayoutStructure>(
        &self,
        p: Vec2,
        tolerance: f32,
        context: &T::Context<'_>,
    ) -> Option<T>;
    fn try_pick<T: LayoutStructure>(&self, p: Vec2, context: &T::Context<'_>) -> Option<T>;

    fn get_origin<T: LayoutStructure + LayoutStructureWithOrigin>(
        &self,
        entity: &T,
        context: &T::Context<'_>,
    ) -> Vec2;
}

fn layout(size: &Size) -> LayoutSizing {
    //TODO include Layout on the window_size object
    LayoutSizing::from_page_size(
        Vec2 {
            x: size.scaled_width,
            y: size.scaled_height,
        },
        IDEAL_RATIO,
        IDEAL_WIDTH,
    )
}

impl SaladWindowSize for Size {
    fn get_rect<T: LayoutStructure>(
        &self,
        entity: &T,
        context: &T::Context<'_>,
    ) -> LayoutRectangle {
        let mut rect = layout(self).get_rect(entity, context);

        rect.top_left = Vec2 {
            x: (self.scaled_width * -0.5) + rect.top_left.x,
            y: (self.scaled_height * 0.5) - rect.top_left.y,
        };

        rect.extents.y *= -1.0;

        rect
    }

    fn get_origin<T: LayoutStructure + LayoutStructureWithOrigin>(
        &self,
        entity: &T,
        context: &T::Context<'_>,
    ) -> Vec2 {
        let rect = self.get_rect(entity, context);
        let origin = entity.origin(context, &layout(self));

        match origin {
            Origin::Center => rect.centre(),
            Origin::TopLeft => rect.top_left,
            Origin::CenterLeft => rect.centre_left(),
            Origin::TopCenter => rect.top_centre(),
        }
    }

    fn try_pick<T: LayoutStructure>(&self, p: Vec2, context: &T::Context<'_>) -> Option<T> {
        layout(self).try_pick_entity(p, 1.0, context)
    }

    fn try_pick_with_tolerance<T: LayoutStructure>(
        &self,
        p: Vec2,
        tolerance: f32,
        context: &T::Context<'_>,
    ) -> Option<T> {
        layout(self).try_pick_entity(p, tolerance, context)
    }

    fn scale(&self) -> f32 {
        (self.scaled_width / 4.0).min(self.scaled_height / 8.0)
    }

    fn font_size<T: LayoutStructureWithFont>(&self, entity: &T, context: &T::FontContext) -> f32 {
        layout(self).font_size(entity, context)
    }

    fn tile_size(&self, selfie_mode: &SelfieMode, insets: Insets) -> f32 {
        layout(self)
            .get_size(&LayoutGridTile::default(), &(*selfie_mode, insets))
            .x
    }
}

pub const DEFAULT_WINDOW_WIDTH: f32 = 398f32;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 884f32;

pub const TILE_MULTIPLIER: f32 = 0.9;

pub const BOLD_FONT: &str = "embedded://ws_common/../../assets/fonts/Montserrat-Bold.ttf";
pub const SEMIBOLD_FONT: &str = "embedded://ws_common/../../assets/fonts/Montserrat-SemiBold.ttf";
pub const REGULAR_FONT: &str = "embedded://ws_common/../../assets/fonts/Montserrat-Regular.ttf";

pub const TILE_FONT_PATH: &str = BOLD_FONT;

pub const THEME_FONT_PATH: &str = BOLD_FONT;
pub const BUTTONS_FONT_PATH: &str = SEMIBOLD_FONT;
pub const THEME_INFO_FONT_PATH: &str = SEMIBOLD_FONT;

pub const TIMER_FONT_PATH: &str = SEMIBOLD_FONT;

pub const TUTORIAL_FONT_PATH: &str = SEMIBOLD_FONT;
pub const POPUP_FONT_PATH: &str = REGULAR_FONT;

pub const SOLUTIONS_FONT_PATH: &str = SEMIBOLD_FONT;

pub const WORD_SALAD_LOGO_FONT_PATH: &str = BOLD_FONT;

pub const ICON_FONT_PATH: &str = "embedded://ws_common/../../assets/fonts/ws_icons.ttf";

pub const ICON_BUTTON_SIZE: f32 = 40f32; //40 pixels
pub const TOOLBAR_SIZE: f32 = 40f32; //40 pixels
