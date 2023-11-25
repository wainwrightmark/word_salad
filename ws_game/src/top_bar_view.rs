use crate::prelude::*;
use maveric::transition::speed::LinearSpeed;
use maveric::with_bundle::CanWithBundle;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::WithBundle};
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TopBar;

impl MavericNode for TopBar {
    type Context = ViewContext; //TODO check

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.3;
                let top_bar_font_size = size.font_size::<LayoutTopBarButton>();
                commands.add_child(
                    "Burger",
                    Text2DNode {
                        text: "\u{f0c9}",
                        font_size: top_bar_font_size,
                        color: convert_color(palette::BUTTON_TEXT_COLOR),
                        font: MENU_BUTTON_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&LayoutTopBarButton::MenuBurgerButton, &())
                            .centre()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    )),
                    &(),
                );

                let time_text = match context.4.as_ref() {
                    LevelTime::Started(..) => "00:00".to_string(),
                    LevelTime::Finished { total_seconds } => format_seconds(*total_seconds),
                };

                let time_translation = if context.2.is_level_complete() {
                    size.get_rect(&CongratsLayoutEntity::LevelTime, &())
                        .centre()
                        .extend(crate::z_indices::TOP_BAR_BUTTON)
                } else {
                    size.get_rect(&LayoutTopBarButton::TimeCounter, &())
                        .centre()
                        .extend(crate::z_indices::TOP_BAR_BUTTON)
                };

                let units_per_second = if context.2.is_level_complete() {
                    100.0
                } else {
                    1000.0
                };

                commands.add_child(
                    //todo hide this in congrats mode and have a separate timer only in that mode
                    "TimeCounter",
                    WithBundle {
                        node: Text2DNode {
                            text: time_text,
                            font_size: top_bar_font_size,
                            color: convert_color(palette::BUTTON_TEXT_COLOR),
                            font: MENU_BUTTON_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        }
                        .with_bundle(Transform::from_translation(
                            size.get_rect(&LayoutTopBarButton::TimeCounter, &())
                                .centre()
                                .extend(crate::z_indices::TOP_BAR_BUTTON),
                        )),
                        bundle: TimeCounterMarker,
                    }
                    .with_transition_to::<TransformTranslationLens>(
                        //TODO improve this animation
                        time_translation,
                        LinearSpeed { units_per_second },
                    ),
                    &(),
                );

                commands.add_child(
                    "hints",
                    Text2DNode {
                        text: context.2.hints_used.to_string(),
                        font_size: top_bar_font_size,
                        color: convert_color(palette::BUTTON_TEXT_COLOR),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&LayoutTopBarButton::HintCounter, &())
                            .centre()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    )),
                    &(),
                );
            });
    }
}
