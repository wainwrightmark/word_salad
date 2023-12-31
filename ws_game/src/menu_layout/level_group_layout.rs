use crate::completion::TotalCompletion;


use super::{
     MENU_BUTTON_FONT_SIZE, MENU_BUTTON_SPACING, MENU_BUTTON_WIDTH, MENU_BUTTON_HEIGHT
};
use bevy::math::Vec2;
use ws_core::{
    layout::entities::{IDEAL_HEIGHT, IDEAL_WIDTH, TOP_BAR_HEIGHT},
    LayoutStructure, LayoutStructureWithFont, Spacing,
};
use ws_levels::level_group::LevelGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelGroupLayoutEntity {
    pub index: usize,
}

impl LevelGroupLayoutEntity {
    pub fn get_text(&self, completion: &TotalCompletion, group: &LevelGroup) -> (String, String) {
        let name = self.name(group);

        let sequence = group.get_level_sequence(self.index);

        let complete = completion.get_number_complete(&sequence);
        let total = sequence.level_count();

        let complete = complete.min(total);
        let fraction = format!("{:#}",  fmtastic::VulgarFraction::new(complete, total));

        (name.to_string(), fraction)
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

    fn size(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        Vec2 {
            x: MENU_BUTTON_WIDTH,
            y: super::MENU_BUTTON_HEIGHT,
        }
    }

    fn location(&self, _context: &Self::Context) -> bevy::prelude::Vec2 {
        Vec2 {
            x: (IDEAL_WIDTH - MENU_BUTTON_WIDTH) / 2.,
            y: TOP_BAR_HEIGHT
                + Spacing::Centre.apply(
                    IDEAL_HEIGHT - TOP_BAR_HEIGHT,
                    MENU_BUTTON_HEIGHT + MENU_BUTTON_SPACING,
                    super::MENU_VIRTUAL_CHILDREN,
                    self.index,
                ),
        }
    }

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self> {
        LevelGroupLayoutIter {
            next_index: 0,
            group: *context,
        }
    }
}

impl LayoutStructureWithFont for LevelGroupLayoutEntity {
    fn font_size(&self) -> f32 {
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
