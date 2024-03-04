use std::time::Duration;

use crate::prelude::*;
use bevy::text::Text2dBounds;
use bevy_param_shaders::ShaderBundle;
use itertools::Either;
use maveric::transition::speed::LinearSpeed;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use strum::EnumIs;
use ws_core::layout::entities::*;
use ws_core::Tile;
use ws_core::{font_icons, prelude::*};

pub const TILE_LINGER_SECONDS: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum Selectability {
    Selected,
    Selectable,
    Unselectable,
}

impl Selectability {
    pub fn new(tile: Tile, solution: &Solution) -> Self {
        use Selectability::*;

        match solution.last() {
            Some(last) => {
                if solution.contains(&tile) {
                    Selected
                } else if last.is_adjacent_to(&tile) {
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
            ManualHinted
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
    pub is_level_complete: bool,
    pub pause_type: PauseType,
}

#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum PauseType {
    NotPaused,
    Blank,
    BlankWithPlay,
}

const TILE_SCALE_SPEED: LinearSpeed = LinearSpeed {
    units_per_second: 1.0,
};

#[derive(Debug, NodeContext)]
pub struct GridTilesContext {
    pub chosen_state: ChosenState,
    pub current_level: CurrentLevel,
    pub found_words_state: FoundWordsState,
    pub window_size: MyWindowSize,
    pub video_resource: VideoResource,
    pub daily_challenges: DailyChallenges,
    pub insets: InsetsResource
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for GridTilesContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            current_level: Res::clone(&value.current_level),
            found_words_state: Res::clone(&value.found_words_state),
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
            chosen_state: Res::clone(&value.chosen_state),
            daily_challenges: Res::clone(&value.daily_challenges),
            insets: Res::clone(&value.insets)
        }
    }
}

impl MavericNode for GridTiles {
    type Context = GridTilesContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert((VisibilityBundle::default(), TransformBundle::default()))
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ordered_children_with_node_and_context(|node, context, commands| {
            let Either::Left(level) = context.current_level.level(&context.daily_challenges) else {
                return;
            };

            let selfie_mode = context.video_resource.selfie_mode();
            let insets = context.insets.0;
            let size = context.window_size.as_ref();
            let tile_size = size.tile_size(&selfie_mode, insets);
            let font_size = size.font_size::<LayoutGridTile>(&LayoutGridTile::default(), &());

            if node.is_level_complete {
                if context.chosen_state.is_just_finished && context.found_words_state.is_changed() {
                    let sol_count = context.chosen_state.solution.len();

                    let target_scale = match sol_count {
                        ..=4 => 1.0,
                        5.. => 4.0 / (sol_count as f32),
                    };

                    for (index, tile) in context.chosen_state.solution.iter().cloned().enumerate() {
                        let initial_centre = size
                            .get_rect(&LayoutGridTile(tile), &(selfie_mode, insets))
                            .centre()
                            .extend(crate::z_indices::GRID_TILE);
                        let character = level.grid[tile];
                        let target_centre = size
                            .get_rect(&LayoutAnimatedTile { index }, &((selfie_mode, insets), sol_count))
                            .centre()
                            .extend(crate::z_indices::GRID_TILE);

                        let scale_transition: Transition<TransformScaleLens> = {
                            let target_scale1 = Vec3 {
                                x: target_scale,
                                y: target_scale,
                                z: 1.0,
                            };

                            if target_scale == 1.0 {
                                let transition = TransitionBuilder::default()
                                    .then_set_value(Vec3::ONE)
                                    .then_wait(Duration::from_secs_f32(
                                        congrats::TRANSITION_WAIT_SECS,
                                    ))
                                    .then_tween_with_duration(
                                        Vec3::ZERO,
                                        Duration::from_secs_f32(congrats::TRANSITION_SECS),
                                    );

                                transition.build()
                            } else {
                                let transition = TransitionBuilder::default()
                                    .then_set_value(target_scale1)
                                    .then_ease_with_duration(
                                        target_scale1,
                                        Duration::from_secs_f32(
                                            congrats::TRANSITION_WAIT_SECS * 0.25,
                                        ),
                                        Ease::CubicOut,
                                    )
                                    .then_wait(Duration::from_secs_f32(
                                        congrats::TRANSITION_WAIT_SECS * 0.75,
                                    ))
                                    .then_ease_with_duration(
                                        Vec3::ZERO,
                                        Duration::from_secs_f32(congrats::TRANSITION_SECS),
                                        Ease::CubicOut,
                                    );

                                transition.build()
                            }
                        };

                        let transform_transition: Transition<TransformTranslationLens> = {
                            let transition = TransitionBuilder::default()
                                .then_set_value(initial_centre)
                                .then_ease_with_duration(
                                    target_centre,
                                    Duration::from_secs_f32(congrats::TRANSITION_WAIT_SECS * 0.25),
                                    Ease::CubicInOut,
                                );

                            transition.build()
                        };

                        commands.add_child(
                            tile.inner() as u32,
                            GridTile {
                                tile,
                                character,
                                selectability: Selectability::Selectable,
                                tile_size,
                                font_size,

                                hint_status: HintStatus::Unknown,
                                timer_paused: !node.pause_type.is_not_paused(),
                                is_selfie_mode: context.video_resource.is_selfie_mode,
                            }
                            .with_bundle(Transform::from_translation(initial_centre))
                            .with_transition::<TransformTranslationLens, ()>(
                                initial_centre,
                                transform_transition,
                                (),
                            )
                            .with_transition::<TransformScaleLens, ()>(
                                Vec3::ONE,
                                scale_transition,
                                (),
                            ),
                            &(),
                        );
                    }
                }

                return;
            }

            let solution = context.chosen_state.current_solution();
            let inadvisable_tiles: GridSet = context
                .found_words_state
                .calculate_inadvisable_tiles(solution, level);

            let hint_set = &context.found_words_state.manual_hint_set(level, solution); //TODO this should reveal if a tile is previously hinted

            let inadvisable_tiles = inadvisable_tiles.intersect(&hint_set.negate());

            for (tile, character) in level.grid.enumerate() {
                if character.is_blank() {
                    continue;
                }

                let needed = !context.found_words_state.unneeded_tiles.get_bit(&tile);
                if !needed {
                    continue;
                }

                let selectability = Selectability::new(tile, solution);
                let hint_status =
                    HintStatus::new(tile, selectability, hint_set, &inadvisable_tiles);

                let centre = size.get_rect(&LayoutGridTile(tile), &(selfie_mode, insets)).centre();

                commands.add_child(
                    tile.inner() as u32,
                    GridTile {
                        tile,
                        character: *character,
                        selectability,
                        tile_size,
                        font_size,

                        hint_status,
                        timer_paused: !node.pause_type.is_not_paused(),
                        is_selfie_mode: context.video_resource.is_selfie_mode,
                    }
                    .with_bundle(Transform::from_translation(
                        centre.extend(crate::z_indices::GRID_TILE),
                    )),
                    &(),
                );
            }

            if node.pause_type.is_blank_with_play() {
                commands.add_child(
                    "play_button",
                    Text2DNode {
                        text: font_icons::PLAY,
                        font: ICON_FONT_PATH,
                        font_size: PlayButtonLayoutStructure.font_size(&()),
                        color: if selfie_mode.is_selfie_mode {
                            palette::GRID_LETTER_SELFIE.convert_color()
                        } else {
                            palette::GRID_LETTER_NORMAL.convert_color()
                        },
                        justify_text: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                        text_anchor: bevy::sprite::Anchor::Center,
                        text_2d_bounds: Text2dBounds::UNBOUNDED,
                    }
                    .with_bundle(Transform::from_translation(
                        size.get_rect(&PlayButtonLayoutStructure, &(selfie_mode, insets))
                            .centre()
                            .extend(crate::z_indices::TILE_TEXT),
                    )),
                    &(),
                );
            }
        });
    }

    fn should_recreate(
        &self,
        _previous: &Self,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
    ) -> bool {
        context.current_level.is_changed()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridTile {
    pub tile: Tile,
    pub character: Character,
    pub selectability: Selectability,
    pub hint_status: HintStatus,
    pub tile_size: f32,
    pub font_size: f32,
    pub timer_paused: bool,
    pub is_selfie_mode: bool,
}

impl GridTile {
    fn fill_color(&self) -> Color {
        if self.is_selfie_mode {
            palette::GRID_TILE_FILL_SELFIE.convert_color()
        } else {
            palette::GRID_TILE_FILL_NORMAL.convert_color()
        }
    }

    fn letter_color(is_selfie_mode: bool, is_selected: bool) -> Color {
        if is_selfie_mode {
            palette::GRID_LETTER_SELFIE
        } else if is_selected {
            palette::GRID_LETTER_SELECTED
        } else {
            palette::GRID_LETTER_NORMAL
        }
        .convert_color()
    }

    fn get_letter_node(&self) -> impl MavericNode<Context = ()> {
        let color = Self::letter_color(self.is_selfie_mode, self.selectability.is_selected());

        Text2DNode {
            text: self.character.to_tile_string(),
            font: TILE_FONT_PATH,
            font_size: self.font_size,
            color,
            justify_text: JustifyText::Center,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
            text_2d_bounds: Default::default(),
            text_anchor: Default::default(),
        }
        .with_transition_to::<TextColorLens<0>>(color, 5.0.into(), None)
        .with_bundle(Transform::from_xyz(0.0, 0.0, crate::z_indices::TILE_TEXT))
    }
}

impl MavericNode for GridTile {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        let mut commands = commands.ignore_context();
        commands.insert_static_bundle((VisibilityBundle::default(), GlobalTransform::default()));
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered_children_with_node(|node, commands| {
            let tile_size = node.tile_size;
            let fill = node.fill_color();

            commands.add_child(
                "tile",
                basic_box_node1(
                    tile_size,
                    tile_size,
                    Vec3::new(0.0, 0.0, crate::z_indices::GRID_TILE),
                    fill,
                    0.0,
                ),
                &(),
            );

            if !node.timer_paused {
                commands.add_child("letter", node.get_letter_node(), &());

                if let HintStatus::ManualHinted = node.hint_status {
                    let (count1, count2) = (4.0, 3.0);
                    let seed = node.tile.inner() as f32 * 123.456;

                    commands.add_child(
                        "sparkle",
                        ShaderBundle::<SparkleShader> {
                            parameters: SparkleParams {
                                count1,
                                count2,
                                seed,
                            },
                            transform: Transform {
                                translation: Vec3::Z * 100.0,
                                scale: Vec3::ONE * tile_size * 0.5,
                                ..default()
                            },
                            ..Default::default()
                        },
                        &(),
                    );
                }
            }
        })
    }

    fn on_deleted(&self, commands: &mut ComponentCommands) -> DeletionPolicy {
        let transition = TransitionBuilder::<TransformScaleLens>::default()
            .then_ease(Vec3::ZERO, TILE_SCALE_SPEED, Ease::CircIn)
            .build();

        commands.insert(transition);

        let letter_color = Self::letter_color(self.is_selfie_mode, false);

        if letter_color != Self::letter_color(self.is_selfie_mode, true) {
            commands.modify_children(|child, mut ec| {
                //ec.insert(Transition::<TextColorLens>::SetValue { value: letter_color, next: None });
                if let Some(text) = child.get::<Text>() {
                    let mut text = text.clone();
                    for section in text.sections.iter_mut() {
                        section.style.color = letter_color;
                    }

                    ec.insert(text);
                    ec.remove::<Transition<TextColorLens<0>>>();
                }
            });
        }

        DeletionPolicy::Linger(std::time::Duration::from_secs_f32(TILE_LINGER_SECONDS))
    }
}

#[derive(Debug, PartialEq)]
pub struct PlayButtonLayoutStructure;

impl LayoutStructure for PlayButtonLayoutStructure {
    type Context<'a> = (SelfieMode, Insets);

    fn size(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        GameLayoutEntity::Grid.size(context, sizing)
    }

    fn location(&self, context: &Self::Context<'_>, sizing: &LayoutSizing) -> Vec2 {
        GameLayoutEntity::Grid.location(context, sizing)
    }

    fn iter_all(_context: &Self::Context<'_>) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }
}

impl LayoutStructureWithFont for PlayButtonLayoutStructure {
    type FontContext = ();

    fn font_size(&self, _context: &Self::FontContext) -> f32 {
        160.0
    }
}
