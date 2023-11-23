use crate::prelude::*;
pub use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::RoundedPolygon;
use maveric::{
    transition::speed::{LinearSpeed, ScalarSpeed},
    widgets::text2d_node::Text2DNode,
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
    Selectable,
    Selected,
    Unselectable,
    Inadvisable
}

impl Selectability{
    pub fn tile_fill_color(&self)-> Color{
        match self{
            Selectability::Selectable => convert_color(palette::GRID_TILE_FILL_SELECTABLE),
            Selectability::Selected => convert_color(palette::GRID_TILE_FILL_SELECTED),
            Selectability::Unselectable => convert_color(palette::GRID_TILE_FILL_UNSELECTABLE),
            Selectability::Inadvisable => convert_color(palette::GRID_TILE_FILL_INADVISABLE),
        }
    }
}

maveric::define_lens!(StrokeColorLens, Stroke, Color, color);
maveric::define_lens!(StrokeOptionsLens, Stroke, StrokeOptions, options);
maveric::define_lens!(StrokeOptionsWidthLens, StrokeOptions, f32, line_width);
maveric::define_lens!(FillColorLens, Fill, Color, color);
pub type StrokeWidthLens = Prism2<StrokeOptionsLens, StrokeOptionsWidthLens>;
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
            let selectable_tiles: GridSet = match selected_tile{
                Some(tile) => GridSet::from_iter(tile.iter_adjacent()),
                None => GridSet::ALL,
            };
            let level = context.1.level();
            let inadvisable_tiles: GridSet = context.2.calculate_inadvisable_tiles(&solution, level);

            let hint_set = context.2.auto_hint_set(&level, &solution).union(&context.2.manual_hint_set(&level, &solution));

            let inadvisable_tiles = inadvisable_tiles.intersect(&hint_set.negate());
            for (tile, character) in context.1.level().grid.enumerate() {
                if character.is_blank() {
                    continue;
                }

                let needed = !context.2.unneeded_tiles.get_bit(&tile);
                if !needed {
                    continue;
                }


                let selectability = if Some(&tile) == selected_tile{
                    Selectability::Selected
                } else if inadvisable_tiles.get_bit(&tile){
                    Selectability::Inadvisable
                }else if selectable_tiles.get_bit(&tile){
                    Selectability::Selectable
                }else{
                    Selectability::Unselectable
                };


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

fn make_rounded_square(size: f32, radius: f32) -> RoundedPolygon {
    let a = size * 0.5;
    let m_a = size * -0.5;

    let rectangle = shapes::RoundedPolygon {
        points: vec![
            Vec2 { x: a, y: a },
            Vec2 { x: a, y: m_a },
            Vec2 { x: m_a, y: m_a },
            Vec2 { x: m_a, y: a },
        ],
        radius,
        closed: true,
    };

    rectangle
}

impl MavericNode for GridTile {
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .map_args(|x| &x.centre)
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
            let fill =  node.selectability.tile_fill_color();

            commands.add_child(
                0,
                LyonShapeNode {
                    shape: make_rounded_square(tile_size, tile_size * 0.1),
                    transform: Transform::from_xyz(0.0, 0.0, crate::z_indices::GRID_TILE),
                    fill: Fill::color(convert_color(palette::GRID_TILE_FILL_SELECTABLE)),
                    stroke: Stroke::color(convert_color(palette::GRID_TILE_STROKE)),
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
    type Context = NoContext;
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
                    transform: Transform::from_xyz(0.0, 0.0, crate::z_indices::TILE_TEXT),
                    text: TextNode {
                        text: args.character.to_tile_string(),
                        font: TILE_FONT_PATH,
                        font_size: args.font_size,
                        color: convert_color(palette::GRID_LETTER),
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                },
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
                let auto_hints = context.2.auto_hint_set(level, &solution);

                for tile in (manual_hints.union(&auto_hints)).iter_true_tiles() {
                    let rect = context.3.get_rect(&LayoutGridTile(tile), &());

                    let translation = rect.centre().extend(crate::z_indices::HINT);
                    let tile_size = rect.extents.x;

                    let shape = make_rounded_square(tile_size * 0.8, tile_size * 0.1);

                    let color = convert_color(if manual_hints.get_bit(&tile) {
                        palette::MANUAL_HINT_GLOW
                    } else {
                        palette::AUTO_HINT_GLOW
                    });

                    commands.add_child(
                        tile.inner() as u32,
                        {
                            LyonShapeNode {
                                transform: Transform::from_translation(translation),
                                fill: Fill::color(Color::NONE),
                                stroke: Stroke::new(color.with_a(0.7), tile_size * 0.1),
                                shape,
                            }
                            .with_transition_in::<StrokeColorLens>(
                                color.with_a(0.0),
                                color.with_a(0.7),
                                Duration::from_secs(1),
                            )
                        },
                        &(),
                    );
                }
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordLine {
    pub solution: Solution,
    pub should_hide: bool,
}

impl MavericNode for WordLine {
    type Context = Size;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.advanced(|args, commands| {
            if !args.is_hot() {
                return;
            }
            let size = args.context;

            let (solution, visible) = if args.node.solution.len() > 0 {
                (args.node.solution.as_slice(), !args.node.should_hide)
            } else {
                match args.previous {
                    Some(s) => {
                        if s.should_hide {
                            (args.node.solution.as_slice(), false)
                        } else {
                            (s.solution.as_slice(), false)
                        }
                    }
                    None => (args.node.solution.as_slice(), false),
                }

                // (
                //     args.previous.map(|x| x.solution.as_slice()).unwrap_or_default(),
                //     false,
                // )
            };

            let mut builder = PathBuilder::new();

            for (index, tile) in solution.iter().enumerate() {
                let position = size.get_rect(&LayoutGridTile(*tile), &()).centre();
                if index == 0 {
                    builder.move_to(position);
                    builder.line_to(position);
                } else {
                    builder.line_to(position);
                }
            }

            let mut width = size.get_rect(&GameLayoutEntity::Grid, &()).extents.x * 50. / 320.;

            if !visible {
                //info!("Word line not visible");
                commands.transition_value::<StrokeWidthLens>(
                    50.0,
                    0.0,
                    Some(ScalarSpeed {
                        amount_per_second: 50.0,
                    }),
                );
            } else if args
                .previous
                .is_some_and(|x| !x.solution.is_empty() && !x.should_hide)
            {
                //info!("Word line remains visible");
                commands.remove::<Transition<StrokeWidthLens>>();
            } else {
                //info!("Word line newly visible");
                commands.insert(Transition::<StrokeWidthLens>::new(TransitionStep::new_arc(
                    width,
                    Some(ScalarSpeed {
                        amount_per_second: width,
                    }),
                    NextStep::None,
                )));

                width = 0.0;
            }

            commands.insert(ShapeBundle {
                path: builder.build(),
                spatial: SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    crate::z_indices::WORD_LINE,
                ))),
                ..Default::default()
            });
            commands.insert(Stroke {
                color: convert_color(palette::WORD_LINE_COLOR),
                options: StrokeOptions::default()
                    .with_line_width(width)
                    .with_start_cap(LineCap::Round)
                    .with_end_cap(LineCap::Round)
                    .with_line_join(LineJoin::Round),
            });
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}
