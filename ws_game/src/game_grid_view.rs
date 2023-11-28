use crate::prelude::*;
use bevy_smud::param_usage::ShaderParamUsage;
use bevy_smud::Frame;
use maveric::{
    transition::speed::LinearSpeed, widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle,
};
use strum::EnumIs;

use std::time::Duration;
use ws_core::layout::entities::*;
use ws_core::prelude::*;
use ws_core::Tile;

#[derive(Debug, Clone, PartialEq)]
pub struct GridTile {
    pub tile: Tile,
    pub character: Character,
    pub selectability: Selectability,
    pub hint_status: HintStatus,
    pub tile_size: f32,
    pub font_size: f32,
    pub centre: Vec2,
}

impl GridTile {
    fn fill_color(&self) -> Color {
        match self.hint_status {
            HintStatus::ManualHinted => palette::GRID_TILE_MANUAL_HINTED.convert_color(),
            //HintStatus::AutoHinted => palette::GRID_TILE_STROKE_AUTO_HINTED.convert_color(),
            _ => palette::GRID_TILE_FILL.convert_color(),
        }
    }

    fn border_color(&self) -> Color {
        palette::GRID_TILE_STROKE.convert_color()
    }

    fn border_proportion(&self) -> f32 {
        if self.selectability.is_selected() {
            2. / 36.
        } else {
            1. / 36.
        }
    }
}

/*
TODOS
    add some sparkles

*/

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum Selectability {
    Selected,
    Selectable,
    Unselectable,
}

impl Selectability {
    pub fn new(tile: Tile, selected_tile: Option<&Tile>) -> Self {
        use Selectability::*;
        match selected_tile {
            Some(selected) => {
                if tile == *selected {
                    Selected
                } else if tile.is_adjacent_to(selected) {
                    Selectable
                } else {
                    Unselectable
                }
            }
            None => Selectable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum HintStatus {
    ManualHinted,
    //AutoHinted,
    Advisable,
    Inadvisable,
    Unknown,
}

impl HintStatus {
    pub fn new(
        tile: Tile,
        selectability: Selectability,
        manual_hints: &GridSet,
        inadvisable: &GridSet,
    ) -> Self {
        use HintStatus::*;
        if manual_hints.get_bit(&tile) {
            return ManualHinted;
        } else if inadvisable.get_bit(&tile) {
            return Inadvisable;
        } else if selectability.is_selectable() && !inadvisable.is_empty() {
            return Advisable;
        } else {
            return Unknown;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridTiles {
    pub level_complete: bool,
}

impl MavericNode for GridTiles {
    type Context = ViewContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert((VisibilityBundle::default(), TransformBundle::default()))
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ordered_children_with_node_and_context(|node, context, commands| {
            if node.level_complete {
                return;
            }
            let solution = context.0.current_solution();
            let selected_tile: Option<&geometrid::prelude::Tile<4, 4>> = solution.last();

            let level = context.1.level();
            let inadvisable_tiles: GridSet =
                context.2.calculate_inadvisable_tiles(&solution, level);

            let hint_set = &context.2.manual_hint_set(&level, &solution);

            let inadvisable_tiles = inadvisable_tiles.intersect(&hint_set.negate());
            for (tile, character) in context.1.level().grid.enumerate() {
                if character.is_blank() {
                    continue;
                }

                let needed = !context.2.unneeded_tiles.get_bit(&tile);
                if !needed {
                    continue;
                }

                let selectability = Selectability::new(tile, selected_tile);
                let hint_status =
                    HintStatus::new(tile, selectability, hint_set, &inadvisable_tiles);

                let size = context.3.as_ref();
                let tile_size = size.tile_size();
                let font_size = size.font_size::<LayoutGridTile>();
                let centre = size.get_rect(&LayoutGridTile(tile), &()).centre();

                commands.add_child(
                    tile.inner() as u32,
                    GridTile {
                        tile,
                        character: *character,
                        selectability,
                        tile_size,
                        font_size,
                        centre,
                        hint_status,
                    },
                    &(),
                );
            }
        });
    }
}

impl MavericNode for GridTile {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .map_node(|x| &x.centre)
            .insert_with_node(|n| {
                TransformBundle::from_transform(Transform::from_translation(n.extend(0.0)))
            })
            .ignore_node()
            .advanced(|args, commands| {
                if args.event == SetEvent::Undeleted {
                    commands.remove::<Transition<TransformScaleLens>>();
                }
            })
            .insert(VisibilityBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let tile_size = node.tile_size;
            let fill = node.fill_color();
            let border_color = node.border_color();
            let border_proportion = node.border_proportion();

            commands.add_child(
                "tile",
                box_with_border_node(
                    tile_size,
                    tile_size,
                    Vec3::new(0.0, 0.0, crate::z_indices::GRID_BORDER),
                    fill,
                    border_color,
                    0.1,
                    border_proportion,
                )
                .with_transition_to::<SmudColorLens>(fill, 0.1.into())
                .with_transition_to::<SmudParamLens<4>>(
                    border_proportion,
                    border_proportion.into(),
                ),
                &(),
            );

            commands.add_child(
                "letter",
                GridLetter {
                    character: node.character,
                    font_size: node.font_size,
                },
                context,
            );

            match node.hint_status {
                HintStatus::ManualHinted | HintStatus::Advisable =>
                {
                    let (p0, p1) = if node.hint_status.is_manual_hinted(){
                        (4.0, 3.0)
                    }else{
                        (2.0, 2.0)
                    };
                    let seed = node.tile.inner() as f32 * 123.456;

                    commands.add_child(
                        "sparkle",
                        SmudShapeNode {
                            color: Color::PINK,
                            sfd: ANYWHERE_SHADER_PATH,
                            fill: SPARKLE_SHADER_PATH,
                            frame_size: 1.0,
                            params: [
                                p0,
                                p1,
                                seed,
                                0.0,
                                0.0,
                                0.0,
                                0.0,
                                0.0,
                            ],
                            sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                            fill_param_usage: ShaderParamUsage::from_params(&[0,1,2]),
                        }
                        .with_bundle(Transform {
                            translation: Vec3::Z * 100.0,
                            scale: Vec3::ONE * tile_size * 0.5,
                            ..default()
                        }),
                        context,
                    );
                }
                 _ => {}
            }




        })
    }

    fn on_deleted(&self, commands: &mut ComponentCommands) -> DeletionPolicy {
        commands.insert(Transition::<TransformScaleLens>::new(
            TransitionStep::new_arc(Vec3::ZERO, Some(LinearSpeed::new(1.0)), NextStep::None),
        ));
        DeletionPolicy::Linger(Duration::from_secs(1))
    }
}

#[derive(Debug, PartialEq)]
pub struct GridLetter {
    pub character: Character,
    pub font_size: f32,
}

impl MavericNode for GridLetter {
    type Context = ();
    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|args, commands| {
            commands.add_child(
                0,
                Text2DNode {
                    text: args.character.to_tile_string(),
                    font: TILE_FONT_PATH,
                    font_size: args.font_size,
                    color: palette::GRID_LETTER.convert_color(),
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                }
                .with_bundle(Transform::from_xyz(
                    0.0,
                    0.0,
                    crate::z_indices::TILE_TEXT,
                )),
                &(),
            )
        });
    }
}

// #[derive(Debug, PartialEq)]
// pub struct HintGlows;

// impl MavericNode for HintGlows {
//     type Context = ViewContext;

//     fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
//         commands
//             .ignore_context()
//             .ignore_node()
//             .insert(SpatialBundle::default());
//     }

//     fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
//         commands
//             .ignore_node()
//             .unordered_children_with_context(|context, commands| {
//                 let level = context.1.level();
//                 let solution = context.0.current_solution();
//                 let manual_hints = context.2.manual_hint_set(level, &solution);

//                 for tile in (manual_hints).iter_true_tiles() {
//                     let rect = context.3.get_rect(&LayoutGridTile(tile), &());

//                     let translation = rect.centre().extend(crate::z_indices::HINT);
//                     let tile_size = rect.extents.x;

//                     //let shape = make_rounded_square(tile_size * 0.8, tile_size * 0.1);

//                     let color = palette::MANUAL_HINT_GLOW.convert_color().with_a(0.7);

//                     commands.add_child(
//                         tile.inner() as u32,
//                         box_border_node(
//                             tile_size * 0.8,
//                             tile_size * 0.8,
//                             translation,
//                             color,
//                             0.1,
//                             0.1,
//                         )
//                         .with_transition_in::<SmudColorLens>(
//                             color.with_a(0.0),
//                             color.with_a(0.7),
//                             Duration::from_secs(1),
//                         ),
//                         &(),
//                     );
//                 }
//             });
//     }
// }
