use crate::prelude::*;
use itertools::Itertools;
use maveric::{transition::speed::ScalarSpeed, widgets::text2d_node::Text2DNode};
use std::time::Duration;
use ws_core::Tile;

pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        //commands.add_child("lines", GridLines, &());
        commands.add_child("cells", GridTiles, context);
        commands.add_child("ui", UI, context);
        commands.add_child("lines", WordLine, context);
    }
}

impl_maveric_root!(ViewRoot);

#[derive(Debug, Clone, PartialEq)]
pub struct UI;

pub const BUTTON_FONT_SIZE: f32 = 22.0;
pub const BUTTON_TEXT_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const TEXT_BUTTON_WIDTH: f32 = 140.;
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
    PreviousLevel,
    NextLevel,
    Hint,
}

impl MavericNode for UI {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        let mapped: SetComponentCommands<(), Size> =
            commands.ignore_node().map_context(|x| &x.3 .0);

        mapped.insert_with_context(|size: &Res<Size>| {
            let y = size.ui_top();
            let left = (size.scaled_width - (size.scale() * 4.0)) * 0.5;
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(y),
                    left: Val::Px(left),
                    width: Val::Px(size.scale() * 4.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,

                    ..Default::default()
                },

                ..Default::default()
            }
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
                        font: CURRENT_STRING_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3 .1,
                );

                let title = context.1.level().name.trim().to_string();

                commands.add_child(
                    "title",
                    TextNode {
                        text: title,
                        font_size: 32.0,
                        color: BUTTON_TEXT_COLOR,
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3 .1,
                );

                let hints = context.2.hint_count();
                let hints = format!("Used {hints} hints");

                commands.add_child(
                    "hints",
                    TextNode {
                        text: hints,
                        font_size: 10.0,
                        color: BUTTON_TEXT_COLOR,
                        font: TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    &context.3 .1,
                );

                commands.add_child("words", WordsNode, context);

                commands.add_child("buttons", ButtonsNode, &context.3 .1);
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


                commands.add_child(
                    "prev",
                    ButtonNode {
                        style: TextButtonStyle::default(),
                        visibility: Visibility::Visible,
                        border_color: Color::BLACK,
                        background_color: Color::NONE,
                        marker: ButtonMarker::PreviousLevel,
                        children: (TextNode {
                            text: "Prev",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
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
                            text: "Next",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },),
                    },
                    &context,
                );

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
                            font: BUTTONS_FONT_PATH,
                            alignment: TextAlignment::Center,
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        },),
                    },
                    &context,
                );

                commands.add_child(
                    "hint",
                    ButtonNode {
                        style: TextButtonStyle::default(),
                        visibility: Visibility::Visible,
                        border_color: Color::BLACK,
                        background_color: Color::NONE,
                        marker: ButtonMarker::Hint,
                        children: (TextNode {
                            text: "Hint",
                            font_size: BUTTON_FONT_SIZE,
                            color: BUTTON_TEXT_COLOR,
                            font: BUTTONS_FONT_PATH,
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
                    if character.is_blank(){
                        continue;
                    }
                    let selected = context.0 .0.last() == Some(&tile);
                    let hinted = hint_set.get_bit(&tile);
                    let needed = !context.2.unneeded_tiles.get_bit(&tile);

                    commands.add_child(
                        tile.inner() as u32,
                        GridTile {
                            tile,
                            character: *character,
                            selected,
                            needed,
                            hinted,
                        },
                        &context.3,
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
    pub hinted: bool,
    pub needed: bool,
}

maveric::define_lens!(StrokeColorLens, Stroke, Color, color);
maveric::define_lens!(FillColorLens, Fill, Color, color);

impl MavericNode for GridTile {
    type Context = NC2<Size, AssetServer>;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        let mut commands: SetComponentCommands<GridTile, Size> = commands.map_context(|x| &x.0);

        commands.scope(|x| {
            x.ignore_context()
                .ignore_node()
                .insert(TransformBundle::default())
                .finish()
        });

        commands
            .animate::<TransformTranslationLens>(
                |node, context| context.tile_position(&node.tile).extend(0.0),
                None,
            )
            .ignore_context()
            .map_args(|node| {
                if node.needed {
                    &0.0
                } else {
                    &std::f32::consts::PI
                }
            })
            .animate_on_node::<TransformRotationYLens>(Some(ScalarSpeed {
                amount_per_second: std::f32::consts::PI,
            }))
            .ignore_context()
            .ignore_node()
            .insert(VisibilityBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            commands.add_child(
                0,
                GridBackground {
                    selected: node.selected,
                    hinted: node.hinted,
                },
                &context.0,
            );
            commands.add_child(
                1,
                GridLetter {
                    character: node.character,
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
}

impl MavericNode for GridLetter {
    type Context = NC2<Size, AssetServer>;
    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert((TransformBundle::default(), VisibilityBundle::default()))
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|args, context, commands| {
            let font_size = context.0.tile_font_size();
            commands.add_child(
                0,
                Text2DNode {
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    text: TextNode {
                        text: args.character.to_tile_string(),
                        font: TILE_FONT_PATH,
                        font_size,
                        color: Color::DARK_GRAY,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                },
                &context.1,
            )
        });
    }
}
#[derive(Debug, PartialEq)]
pub struct GridBackground {
    pub selected: bool,
    pub hinted: bool,
}

impl MavericNode for GridBackground {
    type Context = Size;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .insert_with_context(|context| {
                    let tile_size = context.tile_size(); //todo performance
                    let a = tile_size * 0.5;
                    let m_a = a * -1.0;
                    let rectangle = shapes::RoundedPolygon {
                        points: vec![
                            Vec2{
                                x: a,
                                y: a
                            },
                            Vec2 {
                                x: a,
                                y: m_a,
                            },
                            Vec2 {
                                x: m_a,
                                y: m_a,
                            },
                            Vec2 { x: m_a, y: a },
                        ],
                        radius: tile_size * 0.1,
                        closed: true,
                    };

                    (
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&rectangle),
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),

                            ..Default::default()
                        },
                        Stroke::color(Color::DARK_GRAY),
                        Fill::color(Color::GRAY),
                    )
                })
                .finish()
        });

        commands
            .map_args(|x| match (x.hinted, x.selected) {
                (true, true) => &Color::GOLD,
                (true, false) => &Color::YELLOW,
                (false, true) => &Color::ALICE_BLUE,
                (false, false) => &Color::GRAY,
            })
            .ignore_context()
            .animate_on_node::<FillColorLens>(Some(ScalarSpeed {
                amount_per_second: 1.0,
            }));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

#[derive(Debug, PartialEq)]
pub struct WordsNode;

impl MavericNode for WordsNode {
    type Context = NC4<ChosenState, CurrentLevel, FoundWordsState, NC2<Size, AssetServer>>;

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
                for (index, word) in context.1.level().words.iter().enumerate() {
                    let completion = context.2.get_completion(&word.characters);

                    commands.add_child(
                        index as u32,
                        WordNode {
                            word: word.clone(),
                            completion,
                        },
                        &context.3 .1,
                    )
                }
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
                let position = context.3 .0.tile_position(tile);
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
}

#[derive(Debug, PartialEq)]
pub struct WordNode {
    pub word: Word,
    pub completion: Completion,
}

impl MavericNode for WordNode {
    type Context = AssetServer;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.scope(|commands| {
            commands
                .ignore_node()
                .ignore_context()
                .insert(NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Px(10.0)),

                        //border: UiRect::all(Val::Px(1.0)),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::rgb(0.9, 0.9, 0.9)),
                    ..Default::default()
                })
                .finish()
        });

        commands
            .map_args(|x| x.completion.color())
            .ignore_context()
            .animate_on_node::<BackgroundColorLens>(Some(ScalarSpeed::new(1.0)));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node_and_context(|node, context, commands| {
            let text = match node.completion {
                Completion::Incomplete => str::repeat("?", node.word.characters.len()),
                Completion::Hinted(hints) => node
                    .word
                    .text
                    .chars()
                    .take(hints)
                    .chain(std::iter::repeat('?'))
                    .take(node.word.characters.len())
                    .join(""),
                Completion::Complete => node.word.text.to_string(),
            };

            commands.add_child(
                0,
                TextNode {
                    text,
                    font_size: BUTTON_FONT_SIZE,
                    color: BUTTON_TEXT_COLOR,
                    font: SOLUTIONS_FONT_PATH,
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                },
                &context,
            );
        })
    }
}
