use crate::{animated_solutions, prelude::*, z_indices};
use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
use bevy_smud::{ShapeBundle, SmudShaders, SmudShape};
use itertools::Either;
use maveric::transition::speed::calculate_speed;
use maveric::widgets::text2d_node::Text2DNode;
use maveric::with_bundle::CanWithBundle;
use std::time::Duration;
use ws_core::layout::entities::*;
use ws_core::prelude::*;



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
                let Either::Left(level) = context.1.level(&context.9) else {
                    return;
                };
                let words = &level.words;

                for (index, word) in words.iter().enumerate() {
                    let completion = context.2.get_completion(index);
                    let tile = LayoutWordTile(index);
                    let font_size = context.3.font_size::<LayoutWordTile>(&tile);
                    let rect = context.3.get_rect(&tile, words);
                    let level_indices: (u16, u16) = match context.1.as_ref() {
                        CurrentLevel::Fixed { level_index, .. } => (0, *level_index as u16),
                        CurrentLevel::Custom{..} => (1, 0),
                        CurrentLevel::Tutorial { index } => (2, *index as u16),
                        CurrentLevel::DailyChallenge { index } => (3, *index as u16),
                        CurrentLevel::NonLevel(..) => (4, 0),
                    };
                    commands.add_child(
                        (index as u16, level_indices.0, level_indices.1),
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
                Completion::Unstarted => node.word.hidden_text.to_string(),
                Completion::ManualHinted(hints) => node.word.hinted_text(hints).to_uppercase(),

                Completion::Complete => node.word.text.to_uppercase().to_string(),
            };

            let fill_color = node.completion.color();
            let amount_per_second = if node.completion.is_unstarted() {
                100.0
            } else {
                1.0
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
                    text_2d_bounds: Default::default(),
                    text_anchor: Default::default(),
                }
                .with_bundle(Transform::from_translation(text_translation)),
                &(),
            );

            let shape_translation = centre.extend(crate::z_indices::WORD_BACKGROUND);
            let _shape_border_translation = centre.extend(crate::z_indices::WORD_BACKGROUND + 1.0);



            commands.add_child(
                "shape_fill",
                box_node1(
                    node.rect.extents.x.abs(),
                    node.rect.extents.y.abs(),
                    shape_translation,
                    *fill_color,
                    crate::rounding::WORD_BUTTON_NORMAL,
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
                    crate::rounding::WORD_BUTTON_NORMAL,
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
            ButtonInteraction::WordButton(self.tile),
            TransitionBuilder::<SmudParamLens<2>>::default()
                .then_tween(
                    to,
                    calculate_speed::<f32>(
                        &from,
                        &to,
                        Duration::from_secs_f32(animated_solutions::TOTAL_SECONDS),
                    ),
                )
                .build(),
            ScheduledForDeletion {
                remaining: Duration::from_secs_f32(animated_solutions::TOTAL_SECONDS),
            },
        );

        entity_commands.with_children(|x| {
            x.spawn(bundle);
        });
    }
}
