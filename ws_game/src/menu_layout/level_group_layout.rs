use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH, MENU_BUTTON_FONT_SIZE};
use bevy::math::Vec2;
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_ICON_SIZE},
    LayoutStructure, Spacing, LayoutStructureWithFont, LayoutStructureWithStaticText,
};
use ws_levels::level_group::LevelGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelGroupLayout {
    pub index: usize,
}

impl LayoutStructure for LevelGroupLayout {
    type Context = LevelGroup;

    type Iterator = LevelGroupLayoutIter;

    fn pick(point: bevy::prelude::Vec2, context: &Self::Context) -> Option<Self> {
        for x in Self::iter_all(context) {
            if x.rect(context).contains(point) {
                return Some(x);
            }
        }
        return None;
    }

    fn size(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: super::MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context) -> bevy::prelude::Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_ICON_SIZE,
                    MENU_BUTTON_HEIGHT,
                    context.get_sequences().len(),
                    self.index,
                ),
        }
    }

    fn iter_all(context: &Self::Context) -> Self::Iterator {
        LevelGroupLayoutIter {
            next_index: 0,
            group: *context,
        }
    }
}

impl LayoutStructureWithFont for LevelGroupLayout {
    fn font_size() -> f32 {
        MENU_BUTTON_FONT_SIZE
    }
}

pub struct LevelGroupLayoutIter {
    pub next_index: usize,
    pub group: LevelGroup,
}

impl Iterator for LevelGroupLayoutIter {
    type Item = LevelGroupLayout;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.group.get_sequences().len() {
            return None;
        } else {
            let next = LevelGroupLayout {
                index: self.next_index,
            };
            self.next_index += 1;
            Some(next)
        }
    }
}


impl LayoutStructureWithStaticText for LevelGroupLayout{
    fn text(&self,context: &Self::Context ) -> &'static str {
        context.get_sequences().get(self.index).map(|x|x.name()) .unwrap_or("Unknown")
    }
}