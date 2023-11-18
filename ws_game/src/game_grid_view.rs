use crate::prelude::*;
pub use bevy_prototype_lyon::prelude::*;
use maveric::{
    transition::speed::{LinearSpeed, ScalarSpeed},
    widgets::text2d_node::Text2DNode,
};
use std::time::Duration;
use ws_core::Tile;
#[derive(Debug, Clone, PartialEq)]
pub struct GridTile {
    pub tile: Tile,
    pub character: Character,
    pub selected: bool,
    pub hinted: bool,
    pub needed: bool,
    pub tile_size: f32,
    pub font_size: f32,
    pub centre: Vec2,
}

maveric::define_lens!(StrokeColorLens, Stroke, Color, color);
maveric::define_lens!(FillColorLens, Fill, Color, color);

#[derive(Debug, Clone, PartialEq)]
pub struct GridTiles;

impl MavericNode for GridTiles {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert((VisibilityBundle::default(), TransformBundle::default()))
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .ordered_children_with_context(|context, commands| {
                let hint_set = context.2.hint_set();
                for (tile, character) in context.1.level().grid.enumerate() {
                    if character.is_blank() {
                        continue;
                    }
                    let selected = context.0 .0.last() == Some(&tile);
                    let hinted = hint_set.get_bit(&tile);
                    let needed = !context.2.unneeded_tiles.get_bit(&tile);
                    let size = context.3 .0.as_ref();
                    let tile_size = size.tile_size();
                    let font_size = size.tile_font_size();
                    let centre = size.get_rect(&LayoutTile(tile) ).centre();

                    commands.add_child(
                        tile.inner() as u32,
                        GridTile {
                            tile,
                            character: *character,
                            selected,
                            needed,
                            hinted,
                            tile_size,
                            font_size,
                            centre,
                        },
                        &context.3 .1,
                    );
                }
            });
    }
}

impl MavericNode for GridTile {
    type Context = AssetServer;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {

        commands.scope(|x| {
            x.ignore_context()
                .map_args(|x| &x.centre)
                .insert_with_node(|n| {
                    TransformBundle::from_transform(Transform::from_translation(n.extend(0.0)))
                })
                .finish()
        });

        commands
            .ignore_context()
            .map_args(|node| if node.needed { &Vec3::ONE } else { &Vec3::ZERO })
            .animate_on_node::<TransformScaleLens>(Some(LinearSpeed {
                units_per_second: 1.0,
            }))
            .ignore_context()
            .ignore_node()
            .insert(VisibilityBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let tile_size = node.tile_size;
            let a = tile_size * 0.5;
            let m_a = tile_size * -0.5;

            let rectangle = shapes::RoundedPolygon {
                points: vec![
                    Vec2 { x: a, y: a },
                    Vec2 { x: a, y: m_a },
                    Vec2 { x: m_a, y: m_a },
                    Vec2 { x: m_a, y: a },
                ],
                radius: tile_size * 0.1,
                closed: true,
            };

            let fill = match (node.hinted, node.selected) {
                (true, true) => Color::GOLD,
                (true, false) => Color::YELLOW,
                (false, true) => Color::ALICE_BLUE,
                (false, false) => Color::GRAY,
            };

            commands.add_child(
                0,
                LyonShapeNode {
                    shape: rectangle,
                    transform: Transform::from_xyz(0.0, 0.0, crate::z_indices::GRID_TILE),
                    fill: Fill::color(Color::GRAY),
                    stroke: Stroke::color(Color::DARK_GRAY),
                }
                .with_transition_to::<FillColorLens>(
                    fill,
                    ScalarSpeed {
                        amount_per_second: 1.0,
                    },
                ),
                &(),
            );

            commands.add_child(
                1,
                GridLetter {
                    character: node.character,
                    font_size: node.font_size,
                },
                &context,
            );
        })
    }

    fn on_deleted(&self, commands: &mut ComponentCommands) -> DeletionPolicy {
        commands.insert(Transition::<FillColorLens>::new(TransitionStep::new_arc(
            Color::GRAY,
            Some(ScalarSpeed::new(1.0)),
            NextStep::None,
        )));
        DeletionPolicy::Linger(Duration::from_secs(1))
    }
}

#[derive(Debug, PartialEq)]
pub struct GridLetter {
    pub character: Character,
    pub font_size: f32,
}

impl MavericNode for GridLetter {
    type Context = AssetServer;
    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert((TransformBundle::default(), VisibilityBundle::default()))
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            commands.add_child(
                0,
                Text2DNode {
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    text: TextNode {
                        text: args.character.to_tile_string(),
                        font: TILE_FONT_PATH,
                        font_size: args.font_size,
                        color: Color::DARK_GRAY,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                },
                &context,
            )
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordLine;

impl MavericNode for WordLine {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().insert_with_context(|context| {
            let mut builder = PathBuilder::new();

            for (index, tile) in context.0 .0.iter().enumerate() {
                let position = context
                    .3
                     .0
                    .get_rect(&LayoutTile(*tile))
                    .centre();
                if index == 0 {
                    builder.move_to(position);
                } else {
                    builder.line_to(position);
                }
            }

            let color = Color::rgba(0.9, 0.25, 0.95, 0.9);

            (
                ShapeBundle {
                    path: builder.build(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.5)),
                    ..Default::default()
                },
                Stroke {
                    color,
                    options: StrokeOptions::default()
                        .with_line_width(50.0)
                        .with_start_cap(LineCap::Round)
                        .with_end_cap(LineCap::Round)
                        .with_line_join(LineJoin::Round),
                },
            )
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }

    fn on_deleted(&self, commands: &mut ComponentCommands) -> DeletionPolicy {
        commands.transition_value::<StrokeColorLens>(
            Color::rgba(0.9, 0.25, 0.95, 0.9),
            Color::rgba(0.9, 0.25, 0.95, 0.0),
            Some(ScalarSpeed {
                amount_per_second: 1.0,
            }),
        );
        DeletionPolicy::Linger(Duration::from_secs_f32(1.0))
    }
}
