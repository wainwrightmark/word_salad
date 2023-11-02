use maveric::transition::speed::ScalarSpeed;
use std::time::Duration;
use ws_core::Tile;
use crate::{paths::get_path, prelude::*};

pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, AssetServer>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //commands.add_child("lines", GridLines, &());
        commands.add_child("cells", GridTiles, context);
        commands.add_child("ui", UI, context);
        commands.add_child("lines", WordLine, &context.0);

    }
}

impl_maveric_root!(ViewRoot);

#[derive(Debug, Clone, PartialEq)]
pub struct UI;

pub const FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";

pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const TEXT_BUTTON_WIDTH: f32 = 180.;
pub const TEXT_BUTTON_HEIGHT: f32 = 30.;
pub const UI_BORDER_WIDTH: Val = Val::Px(3.0);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextButtonStyle;
impl IntoBundle for TextButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(TEXT_BUTTON_WIDTH),
            height: Val::Px(TEXT_BUTTON_HEIGHT),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            border: UiRect::all(UI_BORDER_WIDTH),

            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, PartialEq, Component)]
pub enum ButtonMarker {
    Reset,
    NextLevel,
}

impl MavericNode for UI {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, AssetServer>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                top: Val::Px(UI_TOP_LEFT.y),
                left: Val::Px(UI_TOP_LEFT.x),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let level = context.1.level();
                let current_string: String = context
                    .0
                     .0
                    .iter()
                    .map(|tile| level.grid[*tile])
                    .map(|c| c.as_char())
                    .collect();
                let current_string: String = format!("{current_string:^24}");

                commands.add_child(
                    "current",
                    TextNode {
                        text: current_string,
                        font_size: 32.0,
                        color: BUTTON_TEXT_COLOR,
                        font: FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3,
                );

                commands.add_child("buttons", ButtonsNode, &context.3);

                let title = context.1.level().name.clone();

                commands.add_child(
                    "title",
                    TextNode {
                        text: title,
                        font_size: 32.0,
                        color: BUTTON_TEXT_COLOR,
                        font: FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3,
                );

                commands.add_child("words", WordsNode, context);
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonsNode;

impl MavericNode for ButtonsNode {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                commands.add_child(
                    "reset",
                    ButtonNode {
                        style: TextButtonStyle::default(),
                        visibility: Visibility::Visible,
                        border_color: Color::BLACK,
                        background_color: Color::NONE,
                        marker: ButtonMarker::Reset,
                        children: (TextNode {
                            text: "Reset",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },),
                    },
                    &context,
                );

                commands.add_child(
                    "next",
                    ButtonNode {
                        style: TextButtonStyle::default(),
                        visibility: Visibility::Visible,
                        border_color: Color::BLACK,
                        background_color: Color::NONE,
                        marker: ButtonMarker::NextLevel,
                        children: (TextNode {
                            text: "Next Level",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },),
                    },
                    &context,
                );
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridTiles;

impl MavericNode for GridTiles {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, AssetServer>;

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
            .unordered_children_with_context(|context, commands| {
                for (tile, character) in context.1.level().grid.enumerate() {
                    let selected = context.0 .0.last() == Some(&tile);

                    let needed = !context.2.unneeded_tiles.get_bit(&tile);

                    commands.add_child(
                        tile.inner() as u32,
                        GridTile {
                            tile,
                            character: *character,
                            selected,
                            needed,
                        },
                        &(),
                    );
                }
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridTile {
    pub tile: Tile,
    pub character: Character,
    pub selected: bool,
    pub needed: bool,
}

maveric::define_lens!(StrokeColorLens, Stroke, Color, color);
maveric::define_lens!(FillColorLens, Fill, Color, color);

impl MavericNode for GridTile {
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.advanced(|args, commands| {
            if !args.is_hot() {
                return;
            }
            let node = args.node;

            let translation =
                (node.tile.get_north_west_vertex().get_center(SCALE) + GRID_TOP_LEFT).extend(0.0);

            let rotation_y = if args.node.needed {
                0.0
            } else {
                std::f32::consts::PI
            };

            if commands.get::<ComputedVisibility>().is_none() {
                commands.insert(VisibilityBundle {
                    visibility: Visibility::Visible,
                    ..Default::default()
                });
            }

            if commands.get::<GlobalTransform>().is_none() {
                commands.insert(TransformBundle::from_transform(Transform {
                    translation,
                    rotation: Quat::from_rotation_y(rotation_y),
                    scale: Vec3::ONE,
                }));
            }

            commands.transition_value::<TransformRotationYLens>(
                rotation_y,
                rotation_y,
                Some(ScalarSpeed {
                    amount_per_second: std::f32::consts::PI,
                }),
            );
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|node, commands| {
            commands.add_child(
                0,
                GridBackground {
                    selected: node.selected,
                },
                &(),
            );
            commands.add_child(
                1,
                GridLetter {
                    character: node.character,
                },
                &(),
            );
        })
    }

    fn on_deleted(&self, commands: &mut ComponentCommands) -> DeletionPolicy {
        commands.insert(Transition::<FillColorLens>::new(TransitionStep::new_arc(
            Color::GRAY.with_a(0.0),
            Some(ScalarSpeed::new(1.0)),
            NextStep::None,
        )));
        DeletionPolicy::Linger(Duration::from_secs(1))
    }
}

#[derive(Debug, PartialEq)]
pub struct GridLetter {
    pub character: Character,
}

impl MavericNode for GridLetter {
    type Context = NoContext;
    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .insert_with_node(|node| {
                (
                    ShapeBundle {
                        path: Path(get_path(&node.character).0.clone()),
                        transform: Transform::from_xyz(0.0, 0.0, 1.0),
                        ..Default::default()
                    },
                    Stroke::color(Color::DARK_GRAY),
                    Fill::color(Color::GREEN),
                    Transition::<FillColorLens>::new(TransitionStep::new_arc(
                        Color::DARK_GRAY,
                        Some(ScalarSpeed::new(1.0)),
                        NextStep::None,
                    )),
                )
            })
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}
#[derive(Debug, PartialEq)]
pub struct GridBackground {
    pub selected: bool,
}

impl MavericNode for GridBackground {
    type Context = NoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        const RECTANGLE: shapes::Rectangle = shapes::Rectangle {
            extents: Vec2 {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            origin: RectangleOrigin::Center,
        };
        commands
            .insert_with_node(|node| {
                (
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&RECTANGLE),
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),

                        ..Default::default()
                    },
                    Stroke::color(Color::BLACK),
                    Fill::color(if node.selected {
                        Color::BLUE
                    } else {
                        Color::GRAY
                    }),
                )
            })
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, AssetServer>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_context().ignore_node().insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                for (index, word) in context.1.level().words_map.values().enumerate() {
                    let complete = context.2.found.contains(&word.characters);
                    commands.add_child(
                        index as u32,
                        WordNode {
                            word: word.clone(),
                            complete,
                        },
                        &context.3,
                    )
                }
            });
    }
}

#[derive(Debug, PartialEq)]
pub struct WordLine;

impl MavericNode for WordLine {
    type Context = ChosenState;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().insert_with_context(|context| {
            let mut builder = PathBuilder::new();

            for (index, tile) in context.0.iter().enumerate() {
                let location = tile.get_center(SCALE) + GRID_TOP_LEFT - (TILE_SIZE * 0.5);
                if index == 0 {
                    builder.move_to(location);
                } else {
                    builder.line_to(location);
                }
            }

            let color = choose_line_color(0);

            (
                ShapeBundle {
                    path: builder.build(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
                    ..Default::default()
                },
                Stroke {
                    color,
                    options: StrokeOptions::default()
                        .with_line_width(10.0)
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
}

fn choose_line_color(index: usize) -> Color {
    const SATURATIONS: [f32; 2] = [0.9, 0.28];
    const LIGHTNESSES: [f32; 2] = [0.28, 0.49];

    const PHI_CONJUGATE: f32 = 0.618_034;

    let hue = 360. * (((index as f32) * PHI_CONJUGATE) % 1.);
    let lightness: f32 = LIGHTNESSES[index % LIGHTNESSES.len()];
    let saturation: f32 =
        SATURATIONS[(index % (LIGHTNESSES.len() * SATURATIONS.len())) / SATURATIONS.len()];

    let alpha = 0.5;
    Color::hsla(hue, saturation, lightness, alpha)
}

#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub word: Word,
    pub complete: bool,
}

impl MavericNode for WordNode {
    type Context = AssetServer;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),

                ..Default::default()
            },
            border_color: BorderColor(Color::DARK_GRAY),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let text = if node.complete {
                node.word.text.to_string()
            } else {
                str::repeat("?", node.word.characters.len())
            };
            commands.add_child(
                "score",
                TextNode {
                    text,
                    font_size: BUTTON_FONT_SIZE,
                    color: BUTTON_TEXT_COLOR,
                    font: FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                },
                &context,
            );
        })
    }
}
