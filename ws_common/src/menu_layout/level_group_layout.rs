use crate::{completion::SequenceCompletion, prelude::BUTTONS_FONT_PATH, view::MenuContextWrapper};

use ws_core::{layout::entities::*, palette, LayoutStructureDoubleTextButton};
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

impl MenuButtonsLayout for LevelGroupLayoutEntity {
    type Context = LevelGroup;

    fn index(&self) -> usize {
        self.index
    }

    const FONT_SIZE_SMALL: bool = true;

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self> {
        (0..context.get_sequences().len()).map(|x| Self { index: x })
    }

    fn count(context: &Self::Context) -> usize {
        context.get_sequences().len()
    }
}

impl LayoutStructureDoubleTextButton for LevelGroupLayoutEntity {
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
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
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

    fn is_disabled(
        &self,
        _context: &Self::Context<'_>,
        _text_context: &Self::TextContext<'_>,
    ) -> bool {
        false
    }
}
