use crate::prelude::*;
use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
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

                commands.add_child(
                    "Burger",
                    Text2DNode {
                        text: "\u{f0c9}",
                        font_size: size
                            .font_size::<LayoutTopBarButton>(&LayoutTopBarButton::WordSaladButton),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
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

                let hints_rect = size.get_rect(&LayoutTopBarButton::HintCounter, &());
                let hint_font_size =
                    size.font_size::<LayoutTopBarButton>(&LayoutTopBarButton::WordSaladButton);

                commands.add_child(
                    "hints",
                    Text2DNode {
                        text: context.6.hints_remaining.to_string(),
                        font_size: hint_font_size,
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: BUTTONS_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        hints_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                    )),
                    &(),
                );

                commands.add_child(
                    "hints_box",
                    SmudShapeNode {
                        color: palette::HINT_COUNTER_COLOR.convert_color(),
                        sfd: CIRCLE_SHADER_PATH,
                        fill: SIMPLE_FILL_SHADER_PATH,
                        frame_size: 1.0,
                        f_params: [0.0; SHAPE_F_PARAMS],
                        u_params: [0; SHAPE_U_PARAMS],
                        sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                        fill_param_usage: ShaderParamUsage::NO_PARAMS,
                    }
                    .with_bundle(Transform {
                        translation: (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
                            .extend(crate::z_indices::TOP_BAR_BUTTON - 1.0),
                        scale: Vec3::ONE * hints_rect.width() * 0.4,
                        rotation: Default::default(),
                    }),
                    &(),
                );

                const SPARKLE_FILL_PARAMETERS: &[ShaderParameter] = &[
                    ShaderParameter::f32(0),
                    ShaderParameter::f32(1),
                    ShaderParameter::f32(2),
                ];

                commands.add_child(
                    "hints_sparkle",
                    SmudShapeNode {
                        color: palette::HINT_COUNTER_COLOR.convert_color(),
                        sfd: CIRCLE_SHADER_PATH,
                        fill: SPARKLE_SHADER_PATH,
                        frame_size: 0.9,
                        f_params: [3.0, 2.0, 56789.0, 0.0, 0.0, 0.0],
                        u_params: [0; SHAPE_U_PARAMS],
                        sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                        fill_param_usage: ShaderParamUsage(SPARKLE_FILL_PARAMETERS),
                    }
                    .with_bundle(Transform {
                        translation: (hints_rect.centre() + (Vec2::X * hint_font_size * 0.03))
                            .extend(crate::z_indices::TOP_BAR_BUTTON - 0.5),
                        scale: Vec3::ONE * hints_rect.width() * 0.4,
                        rotation: Default::default(),
                    }),
                    &(),
                );

                commands.add_child(
                    //todo hide this in congrats mode and have a separate timer only in that mode
                    "Word Salad Logo text",
                    Text2DNode {
                        text: "Word Salad",
                        font_size: size
                            .font_size::<LayoutTopBarButton>(&LayoutTopBarButton::WordSaladButton),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: WORD_SALAD_LOGO_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle((Transform::from_translation(
                        size.get_rect(&LayoutTopBarButton::WordSaladButton, &())
                            .centre()
                            .extend(crate::z_indices::TOP_BAR_BUTTON),
                    ),)),
                    &(),
                );

                // if context.5.is_closed() && !context.2.is_level_complete() {

                // }
            });
    }
}
