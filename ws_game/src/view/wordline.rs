use crate::{prelude::*, z_indices};
use bevy_param_shaders::prelude::*;
use bitfield_struct::bitfield;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

#[derive(Debug, PartialEq)]
pub struct WordLine {
    pub solution: Solution,
    pub should_hide: bool,
    pub close_to_solution: bool,
}

impl MavericNode for WordLine {
    type Context = MyWindowSize;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle((VisibilityBundle::default(), GlobalTransform::default()));
        commands.advanced(|args, commands| {
            if !args.is_hot() {
                return;
            }

            const DEFAULT_WIDTH: f32 = 0.15;

            let solution: &[Tile];
            let should_hide: bool;
            let final_segment_length: usize;

            if !args.node.solution.is_empty() {
                {
                    should_hide = args.node.should_hide;
                    final_segment_length = args.node.solution.len();
                    match args.previous {
                        Some(previous) => {
                            if previous.solution.len() > args.node.solution.len()
                                && !previous.should_hide
                                && previous.solution.starts_with(&args.node.solution)
                            {
                                solution = previous.solution.as_slice();
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
                should_hide = true;

                match args.previous {
                    Some(previous) => {
                        if previous.should_hide {
                            // The previous was hidden so we should start again
                            solution = args.node.solution.as_slice();
                            final_segment_length = args.node.solution.len();
                        } else {
                            solution = previous.solution.as_slice();
                            final_segment_length = previous.solution.len();
                        }
                    }
                    None => {
                        solution = args.node.solution.as_slice();
                        final_segment_length = args.node.solution.len();
                    }
                }
            };

            let rect = args.context.get_rect(&GameLayoutEntity::Grid, &());

            let scale = rect.extents.x.abs() * 0.5;
            let line_width = if should_hide {
                commands.transition_value::<WordLineWidthLens>(
                    -DEFAULT_WIDTH,
                    (DEFAULT_WIDTH * 1.2).into(),
                )
            } else if args
                .previous
                .is_some_and(|x| !x.solution.is_empty() && !x.should_hide)
            {
                if args.node.close_to_solution {
                    let cycle = TransitionBuilder::<WordLineWidthLens>::default()
                        .then_tween(DEFAULT_WIDTH * 1.05, 0.03.into())
                        .then_tween(DEFAULT_WIDTH, 0.03.into())
                        .build_loop();

                    commands.insert(cycle)
                } else {
                    commands.remove::<Transition<WordLineWidthLens>>();
                }
                DEFAULT_WIDTH
            } else {
                //info!("Word line newly visible");
                commands.insert(
                    TransitionBuilder::<WordLineWidthLens>::default()
                        .then_tween(DEFAULT_WIDTH, (DEFAULT_WIDTH * 4.0).into())
                        .build(),
                );

                DEFAULT_WIDTH * 0.25
            };

            let u_params = solution_to_u32s(solution);

            let final_segment_length = final_segment_length as f32;
            let speed = 4.0;

            let progress = commands
                .transition_value::<WordLineProgressLens>(final_segment_length, speed.into())
                .min(final_segment_length + 1.0) // don't go more than half past the last tile
                .min(solution.len() as f32) //don't show more tiles than we know
                .max(final_segment_length - 0.75); //always be relatively close to the end

            commands.insert(ShaderBundle::<WordLineShader> {
                parameters: WordLineParams {
                    line_width,
                    progress,
                    points1: u_params[0].into(),
                    points2: u_params[1].into(),
                    points3: u_params[2].into(),
                    points4: u_params[3].into(),
                },
                transform: Transform {
                    translation: rect.centre().extend(z_indices::WORD_LINE),
                    scale: Vec3::ONE * scale,
                    ..default()
                },
                ..default()
            });
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

fn solution_to_u32s(solution: &[Tile]) -> [WordLinePoints; 4] {
    // let first = solution.last()?;
    let mut block1 = WordLinePoints::default();
    let mut block2 = WordLinePoints::default();
    let mut block3 = WordLinePoints::default();
    let mut block4 = WordLinePoints::default();

    let mut iter = solution.iter().map(|x| x.inner());

    block1 = block1.with_p0(iter.next().unwrap_or_default());
    block1 = block1.with_p1(iter.next().unwrap_or_default());
    block1 = block1.with_p2(iter.next().unwrap_or_default());
    block1 = block1.with_p3(iter.next().unwrap_or_default());
    block2 = block2.with_p0(iter.next().unwrap_or_default());
    block2 = block2.with_p1(iter.next().unwrap_or_default());
    block2 = block2.with_p2(iter.next().unwrap_or_default());
    block2 = block2.with_p3(iter.next().unwrap_or_default());

    block3 = block3.with_p0(iter.next().unwrap_or_default());
    block3 = block3.with_p1(iter.next().unwrap_or_default());
    block3 = block3.with_p2(iter.next().unwrap_or_default());
    block3 = block3.with_p3(iter.next().unwrap_or_default());
    block4 = block4.with_p0(iter.next().unwrap_or_default());
    block4 = block4.with_p1(iter.next().unwrap_or_default());
    block4 = block4.with_p2(iter.next().unwrap_or_default());
    block4 = block4.with_p3(iter.next().unwrap_or_default());

    //info!("{master:?} {block1:?} {block2:?}");

    [block1, block2, block3, block4]
}

#[bitfield(u32, order = Lsb)]
struct WordLinePoints {
    #[bits(4)]
    p0: u8,

    #[bits(4)]
    p1: u8,

    #[bits(4)]
    p2: u8,

    #[bits(4)]
    p3: u8,

    #[bits(16)]
    buffer: u16, // #[bits(4)]
                 // p4: u8,

                 // #[bits(4)]
                 // p5: u8,

                 // #[bits(4)]
                 // p6: u8,

                 // #[bits(4)]
                 // p7: u8,
}


