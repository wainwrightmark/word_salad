use crate::prelude::*;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

impl MavericNode for CongratsView {
    type Context = ViewContext;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.3;
                let font_size = size.font_size::<CongratsLayoutEntity>();

                let hints_used_text = match context.2.hints_used1 {
                    0 => "No hints used".to_string(),
                    1 => "1 hint used".to_string(),
                    n => format!("{n} hints used"),
                };

                commands.add_child(
                    "hints used",
                    Text2DNode {
                        text: hints_used_text,
                        font_size,
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&CongratsLayoutEntity::HintsUsed, &())
                            .centre()
                            .extend(crate::z_indices::CONGRATS_BUTTON),
                    )),
                    &(),
                );

                // let time_text = match context.4.as_ref() {
                //     LevelTime::Started(..) => "00:00".to_string(),
                //     LevelTime::Finished { total_seconds } => format_seconds(*total_seconds),
                // };
                let top_bar_font_size = size.font_size::<LayoutTopBarButton>();

                // let time_position_initial = size
                //     .get_rect(&LayoutTopBarButton::TimeCounter, &())
                //     .centre()
                //     .extend(crate::z_indices::TOP_BAR_BUTTON);

                // let time_position_final = size
                //     .get_rect(&CongratsLayoutEntity::TimeCounter, &())
                //     .centre()
                //     .extend(crate::z_indices::TOP_BAR_BUTTON);

                // commands.add_child(
                //     //todo hide this in congrats mode and have a separate timer only in that mode
                //     "TimeCounter",
                //     Text2DNode {
                //         text: time_text,
                //         font_size: top_bar_font_size,
                //         color: palette::BUTTON_TEXT_COLOR.convert_color(),
                //         font: MENU_BUTTON_FONT_PATH,
                //         alignment: TextAlignment::Center,
                //         linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                //     }
                //     .with_bundle((TimeCounterMarker,))
                //     .with_transition_in::<TransformTranslationLens>(
                //         time_position_initial,
                //         time_position_final,
                //         Duration::from_millis(500),
                //     ),
                //     &(),
                // );

                commands.add_child(
                    "next level",
                    Text2DNode {
                        text: "Next",
                        font_size,
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&CongratsLayoutEntity::NextButton, &())
                            .centre()
                            .extend(crate::z_indices::CONGRATS_BUTTON),
                    )),
                    &(),
                );

                #[cfg(target_arch = "wasm32")]
                {
                    commands.add_child(
                        "share",
                        Text2DNode {
                            text: "Share",
                            font_size,
                            color: palette::BUTTON_TEXT_COLOR.convert_color(),
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        }
                        .with_bundle(Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::ShareButton, &())
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        )),
                        &(),
                    );
                }
            });
    }
}
