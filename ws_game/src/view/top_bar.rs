use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
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

                let selfie = context.8.selfie_mode().is_selfie_mode;

                commands.add_child(
                    "Burger",
                    Text2DNode {
                        text: "\u{e800}",
                        font_size: size.font_size::<LayoutTopBar>(
                            &LayoutTopBar::MenuBurgerButton,
                            &HintCount::default(),
                        ),
                        color: (if selfie {
                            palette::TOP_BAR_BURGER_SELFIE
                        } else {
                            palette::TOP_BAR_BURGER_NORMAL
                        })
                        .convert_color(),
                        font: ICON_FONT_PATH,
                        alignment: TextAlignment::Left,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: bevy::sprite::Anchor::CenterLeft,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&LayoutTopBar::MenuBurgerButton, &())
                            .centre_left()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    )),
                    &(),
                );

                commands.add_child(
                    "hints",
                    HintsViewNode {
                        hint_state: context.6.clone(),
                    },
                    size,
                );

                commands.add_child(
                    "Word Salad Logo text",
                    Text2DNode {
                        text: "Word Salad",
                        font_size: size.font_size::<LayoutTopBar>(
                            &LayoutTopBar::WordSaladLogo,
                            &HintCount::default(),
                        ),
                        color: (if selfie {
                            palette::TOP_BAR_LOGO_SELFIE
                        } else {
                            palette::TOP_BAR_LOGO_NORMAL
                        })
                        .convert_color(),
                        font: WORD_SALAD_LOGO_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_2d_bounds: Default::default(),
                        text_anchor: bevy::sprite::Anchor::Center,
                    }
                    .with_bundle((Transform::from_translation(
                        size.get_rect(&LayoutTopBar::WordSaladLogo, &())
                            .centre()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    ),)),
                    &(),
                );
            });
    }
}
