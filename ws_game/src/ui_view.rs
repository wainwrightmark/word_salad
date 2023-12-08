use crate::{animated_solutions, prelude::*, z_indices};
use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
use bevy_smud::{ShapeBundle, SmudShaders, SmudShape};
use maveric::transition::speed::calculate_speed;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use std::sync::Arc;
use std::time::Duration;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UI;

impl MavericNode for UI {
    type Context = ViewContext;

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
                let theme = context.1.level().name.trim().to_string();
                let size = &context.3;

                let time_text = match context.4.as_ref() {
                    LevelTime::Started(..) => "00:00".to_string(),
                    LevelTime::Finished { total_seconds } => format_seconds(*total_seconds),
                };

                commands.add_child(
                    "timer",
                    Text2DNode {
                        text: time_text,
                        font_size: size.font_size::<LayoutTextItem>(&LayoutTextItem::Timer),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle((
                        Transform::from_translation(
                            size.get_rect(&LayoutTextItem::Timer, &())
                                .centre()
                                .extend(crate::z_indices::TEXT_AREA_TEXT),
                        ),
                        TimeCounterMarker,
                    )),
                    &(),
                );

                commands.add_child(
                    "theme",
                    Text2DNode {
                        text: theme,
                        font_size: size.font_size::<LayoutTextItem>(&LayoutTextItem::PuzzleTheme),
                        color: palette::BUTTON_TEXT_COLOR.convert_color(),
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&LayoutTextItem::PuzzleTheme, &())
                            .centre()
                            .extend(crate::z_indices::TEXT_AREA_TEXT),
                    )),
                    &(),
                );


            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = ViewContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let words = &context.1.level().words;

                for (index, word) in words.iter().enumerate() {
                    let completion = context.2.get_completion(index);
                    let tile = LayoutWordTile(index);
                    let font_size = context.3.font_size::<LayoutWordTile>(&tile);
                    let rect = context.3.get_rect(&tile, words);
                    let level_index = match context.1.as_ref(){
                        CurrentLevel::Fixed { level_index, .. } => *level_index as u16,
                        CurrentLevel::Custom(_) => 0u16,
                    };
                    commands.add_child(
                        (index as u16, level_index),
                        WordNode {
                            word: word.clone(),
                            tile,
                            completion,
                            rect,
                            font_size,
                        },
                        &(),
                    );
                }
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub tile: LayoutWordTile,
    pub word: DisplayWord,
    pub completion: Completion,
    pub rect: LayoutRectangle,
    pub font_size: f32,
}

impl MavericNode for WordNode {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|node, commands| {
            let text = match node.completion {
                Completion::Unstarted => node.word.hidden_text.clone(),
                Completion::ManualHinted(hints) => node.word.hinted_text(hints),

                Completion::Complete => node.word.text.to_string(),
            };

            let centre = node.rect.centre();

            let text_translation = centre.extend(crate::z_indices::WORD_TEXT);

            commands.add_child(
                "text",
                Text2DNode {
                    text,
                    font_size: node.font_size,
                    color: palette::BUTTON_TEXT_COLOR.convert_color(),
                    font: SOLUTIONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                }
                .with_bundle(Transform::from_translation(text_translation)),
                &(),
            );

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);
            let _shape_border_translation = centre.extend(crate::z_indices::WORD_BACKGROUND + 1.0);

            let fill_color = node.completion.color();
            let amount_per_second = if node.completion.is_unstarted() {
                100.0
            } else {
                0.1
            };

            commands.add_child(
                "shape_fill",
                box_node(
                    node.rect.extents.x.abs(),
                    node.rect.extents.y.abs(),
                    shape_translation,
                    palette::WORD_BACKGROUND_UNSTARTED.convert_color(),
                    0.1,
                )
                .with_bundle(ButtonInteraction::WordButton(node.tile))
                .with_transition_to::<SmudColorLens>(*fill_color, amount_per_second.into()),
                &(),
            );
        })
    }

    fn on_changed(
        &self,
        previous: &Self,
        _context: &<Self::Context as NodeContext>::Wrapper<'_>,
        world: &World,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        if !self.completion.is_complete() || previous.completion.is_complete() {
            return; //this effect is just for newly completed words
        }

        let Some(asset_server) = world.get_resource::<AssetServer>() else {
            return;
        };

        let scale = self.rect.width();
        let translation = self.rect.centre().extend(z_indices::WORD_BACKGROUND + 1.0);
        let rounding = 0.1;
        let height = self.rect.height();

        let from = 1.5;
        let to = -0.5;

        const SDF_PARAMETERS: &[ShaderParameter] =
            &[ShaderParameter::f32(0), ShaderParameter::f32(1)];
        const FILL_PARAMETERS: &[ShaderParameter] = &[
            ShaderParameter::f32(2),
            ShaderParameter::f32(3),
            ShaderParameter::f32(4),
            ShaderParameter::f32(5),
        ];

        let color2 = *previous.completion.color();
        let color1 = *self.completion.color();

        let bundle = ShapeBundle::<SHAPE_F_PARAMS, SHAPE_U_PARAMS> {
            shape: SmudShape {
                //color: Completion::Complete.color().clone(),
                color: color1,
                frame: bevy_smud::Frame::Quad(1.0),
                f_params: [
                    (height / scale),
                    rounding,
                    from,
                    color2.r(),
                    color2.g(),
                    color2.b(),
                ],
                u_params: [0; SHAPE_U_PARAMS],
            },
            shaders: SmudShaders {
                sdf: asset_server.load(BOX_SHADER_PATH),
                fill: asset_server.load(HORIZONTAL_GRADIENT_SHADER_PATH),
                sdf_param_usage: ShaderParamUsage(SDF_PARAMETERS),
                fill_param_usage: ShaderParamUsage(FILL_PARAMETERS),
            },
            transform: Transform {
                translation,
                scale: Vec3::ONE * scale * 0.5,
                ..Default::default()
            },
            ..Default::default()
        };

        let bundle = (
            bundle,
            Transition::<SmudParamLens<2>>::new(TransitionStep::new_arc(
                to,
                Some(calculate_speed::<f32>(
                    &from,
                    &to,
                    Duration::from_secs_f32(animated_solutions::TOTAL_SECONDS),
                )),
                NextStep::None,
            )),
            ScheduledForDeletion {
                timer: Timer::from_seconds(animated_solutions::TOTAL_SECONDS, TimerMode::Once),
            },
        );

        entity_commands.with_children(|x| {
            x.spawn(bundle);
        });
    }
}
