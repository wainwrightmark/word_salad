use crate::{prelude::*, startup, z_indices};
use bevy_param_shaders::prelude::*;
use itertools::Itertools;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

pub struct WordlinePlugin;

impl Plugin for WordlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractToShaderPlugin::<WordLineSegmentShader>::default());
        app.init_resource::<WordLineGlobalTargets>();
        app.insert_resource(WordLineGlobalValues::default());
        app.add_systems(Update, transition_word_line);
    }
}

#[derive(Debug, PartialEq)]
pub struct WordLine {
    pub solution: Solution,
    pub should_hide: bool,
    pub close_to_solution: bool,
    pub selfie_mode: SelfieMode,
    pub special_colors: Option<Vec<BasicColor>>,
    pub insets: Insets,
}

impl MavericNode for WordLine {
    type Context = MyWindowSize;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered(|args, commands| {
            let solution: &[Tile];

            if !args.node.solution.is_empty() {
                {
                    match args.previous {
                        Some(previous) => {
                            if previous.solution.len() > args.node.solution.len()
                                && !previous.should_hide
                                && previous.solution.starts_with(&args.node.solution)
                            {
                                solution = &previous.solution.as_slice()
                                    [0..(args.node.solution.len() + 1)];
                            } else {
                                solution = args.node.solution.as_slice();
                            }
                        }
                        None => {
                            solution = args.node.solution.as_slice();
                        }
                    };
                }
            } else {
                match args.previous {
                    Some(previous) => {
                        if previous.should_hide {
                            // The previous was hidden so we should start again
                            solution = args.node.solution.as_slice();
                        } else {
                            solution = previous.solution.as_slice();
                        }
                    }
                    None => {
                        solution = args.node.solution.as_slice();
                    }
                }
            };

            const SCALE_FACTOR: f32 = ((GRID_GAP / 3.0) + GRID_TILE_SIZE) / GRID_TILE_SIZE;

            if let Ok(tile) = solution.iter().exactly_one() {
                let rect = args.context.get_rect(
                    &LayoutGridTile(*tile),
                    &(args.node.selfie_mode, args.node.insets),
                );
                let color = index_to_color(0, &args.node.special_colors);

                commands.add_child(
                    0,
                    ShaderBundle::<WordLineSegmentShader> {
                        parameters: (
                            WordLineDirection {
                                direction: 0,
                                is_final_segment: true,
                            },
                            ShaderColor { color },
                            ShaderSecondColor { color },
                        ),
                        transform: Transform {
                            translation: rect.centre().extend(z_indices::WORD_LINE),
                            rotation: Default::default(),
                            scale: Vec3::ONE * rect.width() * SCALE_FACTOR,
                        },
                        ..Default::default()
                    },
                    &(),
                );
            } else {
                let mut line_index = usize::MAX;
                let mut last_direction: Option<u32> = None;
                for (index, (from, to)) in solution.iter().tuple_windows().enumerate() {
                    let direction = get_direction(from, to);

                    let rect_f = args.context.get_rect(
                        &LayoutGridTile(*from),
                        &(args.node.selfie_mode, args.node.insets),
                    );
                    let rect_t = args.context.get_rect(
                        &LayoutGridTile(*to),
                        &(args.node.selfie_mode, args.node.insets),
                    );

                    let translation =
                        ((rect_f.centre() + rect_t.centre()) * 0.5).extend(z_indices::WORD_LINE);
                    let color = index_to_color(line_index, &args.node.special_colors);

                    if Some(direction) != last_direction {
                        last_direction = Some(direction);
                        line_index = line_index.wrapping_add(1);
                    }

                    let color2 = index_to_color(line_index, &args.node.special_colors);

                    commands.add_child(
                        index as u32,
                        ShaderBundle::<WordLineSegmentShader> {
                            parameters: (
                                WordLineDirection {
                                    direction,
                                    is_final_segment: index + 2 == solution.len(),
                                },
                                ShaderColor { color },
                                ShaderSecondColor { color: color2 },
                            ),
                            transform: Transform {
                                translation,
                                rotation: Default::default(),
                                scale: Vec3::ONE * rect_f.width() * SCALE_FACTOR,
                            },
                            ..Default::default()
                        },
                        &(),
                    );
                }
            }
        });
    }

    fn on_changed(
        &self,
        previous: &Self,
        _context: &<Self::Context as NodeContext>::Wrapper<'_>,
        _world: &World,
        entity_commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        let should_hide: bool;
        let target_progress: ProgressTarget;

        if self.solution.is_empty() {
            should_hide = true;
            target_progress = ProgressTarget::One;
        } else {
            should_hide = self.should_hide;

            if previous.solution.len() > self.solution.len()
                && !previous.should_hide
                && previous.solution.starts_with(&self.solution)
            {
                target_progress = ProgressTarget::OneThenDecreaseToZero;
            } else {
                target_progress = ProgressTarget::ResetThenIncreaseToOne;
            }
        };

        let target_line_width = if should_hide {
            LineWidthTarget::None
        } else if self.close_to_solution {
            LineWidthTarget::PulseUp
        } else {
            LineWidthTarget::Full
        };
        let targets = WordLineGlobalTargets {
            target_progress,
            target_line_width,
        };

        //info!("Wordline changed {targets:?}");

        entity_commands.commands().insert_resource(targets);
    }

    fn on_deleted(&self, commands: &mut ComponentCommands) -> DeletionPolicy {
        commands.insert_resource(WordLineGlobalTargets {
            target_line_width: LineWidthTarget::None,
            target_progress: ProgressTarget::DecreaseToZero,
        });
        DeletionPolicy::DeleteImmediately
    }
}

fn get_direction(from: &Tile, to: &Tile) -> u32 {
    let horizontal = to.x().cmp(&from.x());
    let vertical = to.y().cmp(&from.y());

    match (horizontal, vertical) {
        (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => 8,
        (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => 7,
        (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => 6,
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => 1,
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => 0,
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => 5,
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => 2,
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => 3,
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => 4,
    }
}

fn index_to_color(index: usize, special_colors: &Option<Vec<BasicColor>>) -> Color {
    //hsl(134, 60%, 41%)

    if let Some(special_colors) = special_colors {
        special_colors[index % special_colors.len()].convert_color()
    } else {
        let hue = (((index as f32) * 20.0) + 134.0) % 360.0;

        Color::hsl(hue, 0.60, 0.41)
    }
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, Default, PartialEq)]
struct WordLineSegmentShader;

impl ExtractToShader for WordLineSegmentShader {
    type Shader = Self;
    type ParamsQuery<'a> = (
        &'a WordLineDirection,
        &'a ShaderColor,
        &'a ShaderSecondColor,
    );
    type ParamsBundle = (WordLineDirection, ShaderColor, ShaderSecondColor);
    type ResourceParams<'w> = Res<'w, WordLineGlobalValues>;

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        let progress = if query_item.0.is_final_segment {
            resource.progress
        } else {
            1.0
        };

        WordLineSegmentShaderParams {
            line_width: resource.line_width,
            direction: query_item.0.direction,

            color1: query_item.1.color.into(),
            color2: query_item.2.color.into(),
            progress,
        }
    }
}

impl ParameterizedShader for WordLineSegmentShader {
    type Params = WordLineSegmentShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "sdf::word_line_segment::sdf(in.pos, in.line_width, in.direction, in.progress)",
            fill_color: "fill::simple::fill(d, mix(in.color1, in.color2, in.progress) , in.pos)",
        }
    }

    fn imports() -> impl Iterator<Item = bevy_param_shaders::prelude::FragmentImport> {
        const WORDLINE_IMPORT: FragmentImport = FragmentImport {
            path: "embedded://ws_common/../../assets/shaders/sdf/word_line_segment.wgsl",
            import_path: "sdf::word_line_segment",
        };

        [WORDLINE_IMPORT, SIMPLE_FILL_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(1.0); // this seems the lowest we can make it (keep in mind the pulsing)

    const UUID: u128 = 0xa68d391613854269a5124561eccd664d;
}

#[derive(Debug, Clone, Component, PartialEq, Default)]
pub struct WordLineDirection {
    pub direction: u32,
    pub is_final_segment: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WordLineSegmentShaderParams {
    pub line_width: f32,
    pub direction: u32,
    pub progress: f32,
    pub color1: LinearRGBA,
    pub color2: LinearRGBA,
}

impl ShaderParams for WordLineSegmentShaderParams {}

const FULL_LINE_WIDTH: f32 = 0.28;
const PULSED_LINE_WIDTH: f32 = FULL_LINE_WIDTH * 1.2;
const ZERO_LINE_WIDTH: f32 = -0.01; //slightly below zero to prevent artifacts
const LINE_WIDTH_DECREASE_SPEED: f32 = FULL_LINE_WIDTH * 1.2;
const LINE_WIDTH_INCREASE_SPEED: f32 = FULL_LINE_WIDTH * 4.0;
const LINE_WIDTH_PULSE_SPEED: f32 = FULL_LINE_WIDTH * 0.5;
const PROGRESS_SPEED: f32 = 4.0;
const RESET_PROGRESS: f32 = 0.25;

#[derive(Debug, Resource, PartialEq)]
struct WordLineGlobalValues {
    pub line_width: f32,
    pub progress: f32,
}

#[derive(Debug, Resource)]
struct WordLineGlobalTargets {
    pub target_line_width: LineWidthTarget,
    pub target_progress: ProgressTarget,
}

impl Default for WordLineGlobalValues {
    fn default() -> Self {
        Self {
            line_width: FULL_LINE_WIDTH,
            progress: 0.0,
        }
    }
}
impl Default for WordLineGlobalTargets {
    fn default() -> Self {
        Self {
            target_line_width: LineWidthTarget::Full,
            target_progress: ProgressTarget::IncreaseToOne,
        }
    }
}

#[derive(Debug, Resource, PartialEq)]
enum LineWidthTarget {
    PulseUp, //TODO track remaining pulses
    PulseDown,
    Full,
    None,
}

#[derive(Debug, Resource, PartialEq)]
enum ProgressTarget {
    One,
    IncreaseToOne,
    DecreaseToZero,
    ResetThenIncreaseToOne,
    OneThenDecreaseToZero,
}

fn transition_word_line(
    mut values: ResMut<WordLineGlobalValues>,
    mut targets: ResMut<WordLineGlobalTargets>,
    time: Res<Time>,
) {
    let progress_change = time.delta_seconds() * PROGRESS_SPEED;
    let mut changed: bool = false;

    let line_width = match targets.target_line_width {
        LineWidthTarget::PulseUp => {
            let width_change = time.delta_seconds() * LINE_WIDTH_PULSE_SPEED;
            let new_width = (values.line_width + width_change).min(PULSED_LINE_WIDTH);

            if new_width >= PULSED_LINE_WIDTH {
                targets.target_line_width = LineWidthTarget::PulseDown;
                changed = true;
            }

            new_width
        }
        LineWidthTarget::PulseDown => {
            let width_change = time.delta_seconds() * -LINE_WIDTH_PULSE_SPEED;
            let new_width = (values.line_width + width_change).max(FULL_LINE_WIDTH);

            if new_width <= FULL_LINE_WIDTH {
                targets.target_line_width = LineWidthTarget::PulseUp;
                changed = true;
            }

            new_width
        }
        LineWidthTarget::Full => {
            let width_change = time.delta_seconds() * LINE_WIDTH_INCREASE_SPEED;
            (values.line_width + width_change).min(FULL_LINE_WIDTH)
        }
        LineWidthTarget::None => {
            let width_change = time.delta_seconds() * LINE_WIDTH_DECREASE_SPEED;
            (values.line_width - width_change).max(ZERO_LINE_WIDTH)
        }
    };

    let progress = match targets.target_progress {
        ProgressTarget::IncreaseToOne => (values.progress + progress_change).min(1.0),
        ProgressTarget::DecreaseToZero => (values.progress - progress_change).max(0.0),
        ProgressTarget::ResetThenIncreaseToOne => {
            changed = true;
            targets.target_progress = ProgressTarget::IncreaseToOne;
            RESET_PROGRESS // + progress_change.min(1.0)
        }
        ProgressTarget::OneThenDecreaseToZero => {
            changed = true;
            targets.target_progress = ProgressTarget::DecreaseToZero;
            1.0
        }
        ProgressTarget::One => {
            changed = true;
            targets.target_progress = ProgressTarget::IncreaseToOne;
            1.0
        }
    };

    let new_values = WordLineGlobalValues {
        line_width,
        progress,
    };

    if values.set_if_neq(new_values) {
        //info!("Transition Word line values {values:?} targets {targets:?}");
        changed = true;
    }

    if changed {
        startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
