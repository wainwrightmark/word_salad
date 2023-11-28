use crate::prelude::*;
use maveric::{
    transition::speed::LinearSpeed, widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle,
};

use std::time::Duration;
use ws_core::layout::entities::*;
use ws_core::prelude::*;
use ws_core::Tile;

#[derive(Debug, Clone, PartialEq)]
pub struct GridTile {
    pub tile: Tile,
    pub character: Character,
    pub selectability: Selectability,
    pub tile_size: f32,
    pub font_size: f32,
    pub centre: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Selectability {
    Selected,
    Advisable,
    Other,
}

impl Selectability {
    pub fn tile_fill_color(&self) -> Color {
        match self {
            Selectability::Advisable => convert_color(palette::GRID_TILE_FILL_ADVISABLE),
            Selectability::Selected => convert_color(palette::GRID_TILE_FILL_SELECTED),
            Selectability::Other => convert_color(palette::GRID_TILE_FILL_OTHER),
        }
    }

    pub fn tile_border_proportion(&self) -> f32 {
        const SELECTED: f32 = 1. / 36.;
        const UNSELECTED: f32 = 1. / 36.;
        match self {
            Selectability::Selected => SELECTED,
            _ => UNSELECTED,
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
            let selectable_tiles: GridSet = match selected_tile {
                Some(tile) => GridSet::from_iter(tile.iter_adjacent()),
                None => GridSet::ALL,
            };

            let selectable_tiles = selectable_tiles.intersect(&context.2.unneeded_tiles.negate());

            let level = context.1.level();
            let inadvisable_tiles: GridSet =
                context.2.calculate_inadvisable_tiles(&solution, level);

            //info!("{} ia {} st", inadvisable_tiles.count(), selectable_tiles.count());

            let show_advisable = !inadvisable_tiles.is_empty()
                && inadvisable_tiles.count() * 2 >= selectable_tiles.count();

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

                let selectability = if Some(&tile) == selected_tile {
                    Selectability::Selected
                } else if show_advisable
                    && !inadvisable_tiles.get_bit(&tile)
                    && selectable_tiles.get_bit(&tile)
                {
                    Selectability::Advisable
                } else {
                    Selectability::Other
                };

                //  else if inadvisable_tiles.get_bit(&tile) {
                //     Selectability::Inadvisable
                // } else if selectable_tiles.get_bit(&tile) {
                //     Selectability::Selectable
                // } else {
                //     Selectability::Unselectable
                // };

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
            let fill = node.selectability.tile_fill_color();

            commands.add_child(
                "tile",
                box_with_border_node(
                    tile_size,
                    tile_size,
                    Vec3::new(0.0, 0.0, crate::z_indices::GRID_BORDER),
                    convert_color(palette::GRID_TILE_FILL_OTHER),
                    convert_color(palette::GRID_TILE_STROKE),
                    0.1,
                    node.selectability.tile_border_proportion(),
                )
                .with_transition_to::<SmudColorLens>(fill, 0.1.into()),
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
                    color: convert_color(palette::GRID_LETTER),
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

#[derive(Debug, PartialEq)]
pub struct HintGlows;

impl MavericNode for HintGlows {
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
                let level = context.1.level();
                let solution = context.0.current_solution();
                let manual_hints = context.2.manual_hint_set(level, &solution);

                for tile in (manual_hints).iter_true_tiles() {
                    let rect = context.3.get_rect(&LayoutGridTile(tile), &());

                    let translation = rect.centre().extend(crate::z_indices::HINT);
                    let tile_size = rect.extents.x;

                    //let shape = make_rounded_square(tile_size * 0.8, tile_size * 0.1);

                    let color = convert_color(palette::MANUAL_HINT_GLOW).with_a(0.7);

                    commands.add_child(
                        tile.inner() as u32,
                        box_border_node(
                            tile_size * 0.8,
                            tile_size * 0.8,
                            translation,
                            color,
                            0.1,
                            0.1,
                        )
                        .with_transition_in::<SmudColorLens>(
                            color.with_a(0.0),
                            color.with_a(0.7),
                            Duration::from_secs(1),
                        ),
                        &(),
                    );
                }
            });
    }
}
