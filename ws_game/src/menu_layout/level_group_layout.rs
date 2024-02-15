use crate::{completion::SequenceCompletion, prelude::BUTTONS_FONT_PATH, view::MenuContextWrapper};

use super::{MENU_BUTTON_HEIGHT, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH};
use bevy::math::Vec2;
use ws_core::{
    layout::entities::*, palette, LayoutSizing, LayoutStructure, LayoutStructureDoubleText,
    LayoutStructureWithFont, Spacing,
};
use ws_levels::level_group::LevelGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelGroupLayoutEntity {
    pub index: usize,
}

impl LevelGroupLayoutEntity {
    pub fn get_text(
        &self,
        completion: &SequenceCompletion,
        group: &LevelGroup,
    ) -> (String, String) {
        let name = self.name(group);

        let sequence = group.get_level_sequence(self.index);

        let num_complete = completion.get_number_complete(&sequence);
        let total = sequence.level_count();

        let complete = num_complete.min(total);
        let fraction = format!("{:#}", fmtastic::VulgarFraction::new(complete, total));

        (name.to_string(), fraction)
    }

    pub fn is_complete(&self, completion: &SequenceCompletion, group: &LevelGroup) -> bool {
        let sequence = group.get_level_sequence(self.index);

        let num_complete = completion.get_number_complete(&sequence);
        let total = sequence.level_count();
        num_complete >= total
    }

    fn name(&self, group: &LevelGroup) -> &'static str {
        group
            .get_sequences()
            .get(self.index)
            .map(|x| x.name())
            .unwrap_or("Unknown")
    }
}

impl LayoutStructure for LevelGroupLayoutEntity {
    type Context<'a> = (SelfieMode, LevelGroup);

    fn size(&self, _context: &Self::Context<'_>, _sizing: &LayoutSizing) -> bevy::prelude::Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: super::MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> bevy::prelude::Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, &context.0))
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - (TOP_BAR_HEIGHT_BASE + extra_top_bar_height(sizing, &context.0)),
                    MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                    super::MENU_VIRTUAL_CHILDREN,
                    self.index,
                ),
        }
    }

    fn iter_all(context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        LevelGroupLayoutIter {
            next_index: 0,
            group: context.1,
        }
    }
}

impl LayoutStructureWithFont for LevelGroupLayoutEntity {
    type FontContext = ();
    fn font_size(&self, _: &()) -> f32 {
        MENU_BUTTON_FONT_SIZE_SMALL
    }
}

pub struct LevelGroupLayoutIter {
    pub next_index: usize,
    pub group: LevelGroup,
}

impl Iterator for LevelGroupLayoutIter {
    type Item = LevelGroupLayoutEntity;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_index.cmp(&self.group.get_sequences().len()) {
            std::cmp::Ordering::Less => {
                let next = LevelGroupLayoutEntity {
                    index: self.next_index,
                };
                self.next_index += 1;
                Some(next)
            }
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => None,
        }
    }
}

impl LayoutStructureDoubleText for LevelGroupLayoutEntity {
    type TextContext<'a> = MenuContextWrapper<'a>;

    fn double_text(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> (String, String) {
        self.get_text(text_context.sequence_completion.as_ref(), &context.1)
    }

    fn left_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn right_font(&self) -> &'static str {
        BUTTONS_FONT_PATH
    }

    fn text_color(
        &self,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> ws_core::prelude::BasicColor {
        palette::MENU_BUTTON_TEXT_REGULAR
    }

    fn fill_color(
        &self,
        background_type: ws_core::prelude::BackgroundType,
        context: &Self::Context<'_>,
        text_context: &Self::TextContext<'_>,
    ) -> ws_core::prelude::BasicColor {
        if self.is_complete(&text_context.sequence_completion, &context.1) {
            background_type.menu_button_complete_fill()
        } else {
            background_type.menu_button_incomplete_fill()
        }
    }
}
