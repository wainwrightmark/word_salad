use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use ws_core::layout::entities::*;
#[derive(Debug, Clone, PartialEq)]
pub struct CongratsView;

//pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

impl MavericNode for CongratsView {
    type Context = NC5<ChosenState, CurrentLevel, FoundWordsState, Size, LevelTime>;

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
                let font_size = size.font_size::<CongratsLayoutEntity>();

                #[cfg(target_arch = "wasm32")]
                {
                    commands.add_child(
                        "share",
                        Text2DNode {
                            text: TextNode {
                                text: "Share",
                                font_size,
                                color: BUTTON_TEXT_COLOR,
                                font: BUTTONS_FONT_PATH,
                                alignment: TextAlignment::Center,
                                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            },
                            transform: Transform::from_translation(
                                size.get_rect(&CongratsLayoutEntity::ShareButton, &())
                                    .centre()
                                    .extend(crate::z_indices::CONGRATS_BUTTON),
                            ),
                        },
                        &(),
                    );
                }

                commands.add_child(
                    "next level",
                    Text2DNode {
                        text: TextNode {
                            text: "Next",
                            font_size,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },
                        transform: Transform::from_translation(
                            size.get_rect(&CongratsLayoutEntity::NextButton, &())
                                .centre()
                                .extend(crate::z_indices::CONGRATS_BUTTON),
                        ),
                    },
                    &(),
                );
            });
    }
}
