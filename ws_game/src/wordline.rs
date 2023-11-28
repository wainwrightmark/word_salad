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
            };

            let rect = args.context.get_rect(&GameLayoutEntity::Grid, &());

            let scale = rect.extents.x.abs() * 0.5;
            let initial_width = if !visible {
                //info!("Word line not visible");
                commands.transition_value::<SmudParamLens<0>>(
                    DEFAULT_WIDTH,
                    DEFAULT_WIDTH * -1.0,
                    Some((DEFAULT_WIDTH * 1.2).into()),
                );
                DEFAULT_WIDTH
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
                commands.insert(Transition::<SmudParamLens<0>>::new(TransitionStep::new_arc(
                    DEFAULT_WIDTH,
                    Some((DEFAULT_WIDTH * 4.0) .into()),
                    NextStep::None,
                )));

                DEFAULT_WIDTH * -1.0
            };

            let asset_server = commands
                .get_res_untracked::<AssetServer>()
                .expect("Wordline should be able to get asset server");

            let fill = asset_server.load(WORD_LINE_FILL_SHADER_PATH);
            let sdf = asset_server.load(WORD_LINE_SHADER_PATH);

            let (arg_x, arg_y, arg_z) = solution_to_u32s(&solution);
            let xu32: u32 = arg_x.into();
            let yu32: u32 = arg_y.into();
            let zu32: u32 = arg_z.into();
            //info!("{xu32} {yu32} {zu32}");

            //info!("Word line {scale}");
            commands.insert((
                SmudShape {
                    color: WORD_LINE_COLOR.convert_color(),
                    fill,
                    sdf,
                    frame: bevy_smud::Frame::Quad(1.0),


                    params: [initial_width,u32_tof32(xu32),u32_tof32(yu32),u32_tof32(zu32), 0.0,0.0,0.0,0.0 ],
                    sdf_param_usage: ShaderParamUsage::from_params(&[0,1,2,3]),
                    fill_param_usage: ShaderParamUsage::NO_PARAMS
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

fn solution_to_u32s(solution: &[Tile]) -> (WordLineMaster, WordLinePoints, WordLinePoints) {
    // let first = solution.last()?;
    let mut master = WordLineMaster::default().with_padding2(1);
    let mut block1 = WordLinePoints::default().with_padding(1);
    let mut block2 = WordLinePoints::default().with_padding(1);

    master = master.with_length(solution.len() as u8);
    let mut iter = solution.iter().map(|x| x.inner());

    master = master.with_p0(iter.next().unwrap_or_default());
    master = master.with_p1(iter.next().unwrap_or_default());

    block1 = block1.with_p0(iter.next().unwrap_or_default());
    block1 = block1.with_p1(iter.next().unwrap_or_default());
    block1 = block1.with_p2(iter.next().unwrap_or_default());
    block1 = block1.with_p3(iter.next().unwrap_or_default());
    block1 = block1.with_p4(iter.next().unwrap_or_default());
    block1 = block1.with_p5(iter.next().unwrap_or_default());
    block1 = block1.with_p6(iter.next().unwrap_or_default());

    block2 = block2.with_p0(iter.next().unwrap_or_default());
    block2 = block2.with_p1(iter.next().unwrap_or_default());
    block2 = block2.with_p2(iter.next().unwrap_or_default());
    block2 = block2.with_p3(iter.next().unwrap_or_default());
    block2 = block2.with_p4(iter.next().unwrap_or_default());
    block2 = block2.with_p5(iter.next().unwrap_or_default());
    block2 = block2.with_p6(iter.next().unwrap_or_default());

    //info!("{master:?} {block1:?} {block2:?}");

    (master, block1, block2)
}

#[bitfield(u32, order = Lsb)]
struct WordLineMaster {
    #[bits(8)]
    pub length: u8,

    #[bits(4)]
    p0: u8,

    #[bits(4)]
    p1: u8,

    #[bits(8)]
    padding1: u8,

    #[bits(8)]
    padding2: u8,
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
    padding: u8,
}

fn u32_tof32(a: u32) -> f32 {
    let r = f32::from_bits(a);
    //info!("{r}");
    r
}
