use crate::completion::TotalCompletion;

use super::{
    MENU_BUTTON_DOUBLE_HEIGHT, MENU_BUTTON_FONT_SIZE, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH,
};
use bevy::math::Vec2;
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_ICON_SIZE},
    LayoutStructure, LayoutStructureWithFont, Spacing,
};
use ws_levels::level_group::LevelGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelGroupLayoutEntity {
    pub index: usize,
}

impl LevelGroupLayoutEntity {
    pub fn get_text(&self, completion: &TotalCompletion, group: &LevelGroup) -> String {
        let name = self.name(group);

        let sequence = group.get_level_sequence(self.index);

        let complete = completion.get_number_complete(&sequence);
        let total = sequence.level_count();

        let complete = complete.min(total);

        format!("{name}\n{complete:3}/{total:3}")
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
            y: super::MENU_BUTTON_DOUBLE_HEIGHT,
        }
    }

    fn location(&self, context: &Self::Context) -> bevy::prelude::Vec2 {

        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: TOP_BAR_ICON_SIZE
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_ICON_SIZE,
                    MENU_BUTTON_DOUBLE_HEIGHT + MENU_BUTTON_SPACING,
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

impl LayoutStructureWithFont for LevelGroupLayoutEntity {
    fn font_size() -> f32 {
        MENU_BUTTON_FONT_SIZE
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
                let next = LevelGroupLayoutEntity{index: self.next_index};
                self.next_index += 1;
                Some(next)
            }
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                return None;
            }
        }
    }
}
