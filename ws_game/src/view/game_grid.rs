use crate::prelude::*;
use bevy_param_shaders::ShaderBundle;
use itertools::Either;
use maveric::transition::speed::LinearSpeed;
use maveric::{widgets::text2d_node::Text2DNode, with_bundle::CanWithBundle};
use strum::EnumIs;
use ws_core::layout::entities::*;
use ws_core::prelude::*;
use ws_core::Tile;

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
    pub level_complete: bool,
}

const TILE_SCALE_SPEED: LinearSpeed = LinearSpeed {
    units_per_second: 1.0,
};

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

            let Either::Left(level) = context.1.level(&context.9) else {
                return;
            };
            let inadvisable_tiles: GridSet = context.2.calculate_inadvisable_tiles(solution, level);

            let hint_set = &context.2.manual_hint_set(level, solution); //TODO this should reveal if a tile is previously hinted

            let selfie_mode = context.8.selfie_mode();

            let inadvisable_tiles = inadvisable_tiles.intersect(&hint_set.negate());
            for (tile, character) in level.grid.enumerate() {
                if character.is_blank() {
                    continue;
                }

                let needed = !context.2.unneeded_tiles.get_bit(&tile);
                if !needed {
                    continue;
                }

                let selectability = Selectability::new(tile, solution);
                let hint_status =
                    HintStatus::new(tile, selectability, hint_set, &inadvisable_tiles);

                let size = context.3.as_ref();
                let tile_size = size.tile_size(&selfie_mode);
                let font_size = size.font_size::<LayoutGridTile>(&LayoutGridTile::default(), &());
                let centre = size.get_rect(&LayoutGridTile(tile), &selfie_mode).centre();

                commands.add_child(
                    tile.inner() as u32,
                    GridTile {
                        tile,
                        character: *character,
                        selectability,
                        tile_size,
                        font_size,

                        hint_status,
                        timer_paused: context.4.is_paused(),
                        is_selfie_mode: context.8.is_selfie_mode,
                    }
                    .with_bundle(Transform::from_translation(
                        centre.extend(crate::z_indices::GRID_TILE),
                    )),
                    &(),
                );
            }
        });
    }

    fn should_recreate(&self, _previous: &Self, context: &<Self::Context as NodeContext>::Wrapper<'_>,)-> bool {
        context.1.is_changed()
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
            palette::MY_WHITE
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
            alignment: TextAlignment::Center,
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
                    0.1,
                ),
                &(),
            );

            commands.add_child("letter", node.get_letter_node(), &());
            if !node.timer_paused {
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
            .then_ease(Vec3::ZERO, TILE_SCALE_SPEED, Ease::BackIn)
            .build();

        commands.insert(transition);

        let letter_color = Self::letter_color(self.is_selfie_mode, false);

        if letter_color != Self::letter_color(self.is_selfie_mode, true){
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


        //info!("Grid Tile on deleted {found_children:?} children found");

        DeletionPolicy::Linger(std::time::Duration::from_secs(1))
    }
}
