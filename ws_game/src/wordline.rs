use crate::{prelude::*, z_indices};
use bevy_smud::SmudShape;

use bevy_smud::param_usage::ShaderParamUsage;
use bitfield_struct::bitfield;
use ws_core::layout::entities::*;
use ws_core::palette::WORD_LINE_COLOR;
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

            if args.node.solution.len() > 0 {
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
            let initial_width = if should_hide {
                commands.transition_value::<SmudParamLens<0>>(
                    -DEFAULT_WIDTH,
                    Some((DEFAULT_WIDTH * 1.2).into()),
                )
            } else if args
                .previous
                .is_some_and(|x| !x.solution.is_empty() && !x.should_hide)
            {
                if args.node.close_to_solution {
                    commands.insert(Transition::<SmudParamLens<0>>::new(
                        TransitionStep::new_cycle(
                            [
                                (DEFAULT_WIDTH * 1.05, 0.03.into()),
                                (DEFAULT_WIDTH, 0.03.into()),
                            ]
                            .into_iter(),
                        ),
                    ))
                } else {
                    commands.remove::<Transition<SmudParamLens<0>>>();
                }
                DEFAULT_WIDTH
            } else {
                //info!("Word line newly visible");
                commands.insert(Transition::<SmudParamLens<0>>::new(
                    TransitionStep::new_arc(
                        DEFAULT_WIDTH,
                        Some((DEFAULT_WIDTH * 4.0).into()),
                        NextStep::None,
                    ),
                ));

                DEFAULT_WIDTH * 0.25
            };

            let asset_server = commands
                .get_res_untracked::<AssetServer>()
                .expect("Wordline should be able to get asset server");

            let fill = asset_server.load(WORD_LINE_FILL_SHADER_PATH);
            let sdf = asset_server.load(WORD_LINE_SHADER_PATH);

            let (points1, points2) = solution_to_u32s(&solution);
            let points1: u32 = points1.into();
            let points2: u32 = points2.into();
            let final_segment_length = final_segment_length as f32;
            let speed = 4.0;

            let segment_length = commands
                .transition_value::<SmudParamLens<3>>(final_segment_length, Some(speed.into()))
                .min(final_segment_length + 1.0) // don't go more than half past the last tile
                .min(solution.len() as f32) //don't show more tiles than we know
                .max(final_segment_length - 0.75); //always be relatively close to the end

            commands.insert((
                SmudShape::<SHAPE_PARAMS> {
                    color: WORD_LINE_COLOR.convert_color(),
                    fill,
                    sdf,
                    frame: bevy_smud::Frame::Quad(1.0),

                    params: [
                        initial_width,
                        u32_tof32(points1),
                        u32_tof32(points2),
                        segment_length,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                    ],
                    sdf_param_usage: ShaderParamUsage::from_params(&[0, 1, 2, 3]),
                    fill_param_usage: ShaderParamUsage::NO_PARAMS,
                },
                Transform {
                    translation: rect.centre().extend(z_indices::WORD_LINE),
                    scale: Vec3::ONE * scale,
                    ..default()
                },
            ));
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.no_children()
    }
}

fn solution_to_u32s(solution: &[Tile]) -> (WordLinePoints, WordLinePoints) {
    // let first = solution.last()?;
    let mut block1 = WordLinePoints::default();
    let mut block2 = WordLinePoints::default();

    let mut iter = solution
        .iter()
        .map(|x| x.inner())
        .chain(std::iter::repeat(1)); //pad with 0b0010 to get around NaN issues

    // master = master.with_p0(iter.next().unwrap_or_default());
    // master = master.with_p1(iter.next().unwrap_or_default());

    block1 = block1.with_p0(iter.next().unwrap_or_default());
    block1 = block1.with_p1(iter.next().unwrap_or_default());
    block1 = block1.with_p2(iter.next().unwrap_or_default());
    block1 = block1.with_p3(iter.next().unwrap_or_default());
    block1 = block1.with_p4(iter.next().unwrap_or_default());
    block1 = block1.with_p5(iter.next().unwrap_or_default());
    block1 = block1.with_p6(iter.next().unwrap_or_default());
    block1 = block1.with_p7(iter.next().unwrap_or_default());

    block2 = block2.with_p0(iter.next().unwrap_or_default());
    block2 = block2.with_p1(iter.next().unwrap_or_default());
    block2 = block2.with_p2(iter.next().unwrap_or_default());
    block2 = block2.with_p3(iter.next().unwrap_or_default());
    block2 = block2.with_p4(iter.next().unwrap_or_default());
    block2 = block2.with_p5(iter.next().unwrap_or_default());
    block2 = block2.with_p6(iter.next().unwrap_or_default());
    block2 = block2.with_p7(iter.next().unwrap_or_default());

    //info!("{master:?} {block1:?} {block2:?}");

    (block1, block2)
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

    #[bits(4)]
    p4: u8,

    #[bits(4)]
    p5: u8,

    #[bits(4)]
    p6: u8,

    #[bits(4)]
    p7: u8,
}

fn u32_tof32(a: u32) -> f32 {
    let r = f32::from_bits(a);
    //info!("{r}");
    r
}
