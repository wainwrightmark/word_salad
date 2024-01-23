use crate::prelude::*;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
use ws_core::prelude::*;


#[derive(Debug, NodeContext)]
pub struct TopBarContext {
    pub window_size: MyWindowSize,
    pub hint_state: HintState,
    pub video_resource: VideoResource,
}

impl<'a, 'w : 'a> From<&'a ViewContextWrapper<'w>> for TopBarContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            hint_state: Res::clone(&value.hint_state),
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TopBar;

impl MavericNode for TopBar {
    type Context = TopBarContext;

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
                let size = &context.window_size;

                let selfie = context.video_resource.selfie_mode().is_selfie_mode;

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
                        size.get_rect(&LayoutTopBar::MenuBurgerButton, &context.video_resource.selfie_mode())
                            .centre_left()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    )),
                    &(),
                );

                commands.add_child(
                    "hints",
                    HintsViewNode {
                        hint_state: context.hint_state.clone(),
                        selfie_mode: context.video_resource.selfie_mode()
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
                        size.get_rect(&LayoutTopBar::WordSaladLogo, &context.video_resource.selfie_mode())
                            .centre()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    ),)),
                    &(),
                );
            });
    }
}
