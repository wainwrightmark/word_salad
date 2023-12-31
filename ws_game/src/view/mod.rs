pub mod congrats;
pub mod game_grid;
pub mod hints;
pub mod level_name;
pub mod level_extra_info;
pub mod menu;
pub mod non_level;
pub mod popup;
pub mod timer;
pub mod top_bar;
pub mod tutorial;
pub mod wordline;
pub mod words;

use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
pub use congrats::*;
pub use game_grid::*;
pub use hints::*;
pub use level_name::*;
pub use level_extra_info::*;
use maveric::with_bundle::CanWithBundle;
pub use menu::*;
pub use non_level::*;
pub use popup::*;
pub use timer::*;
pub use top_bar::*;
pub use tutorial::*;
pub use wordline::*;
pub use words::*;
use ws_core::layout::entities::GameLayoutEntity;

use crate::{completion::TotalCompletion, prelude::*};

pub type ViewContext = (
    ChosenState,
    CurrentLevel,
    FoundWordsState,
    MyWindowSize,
    LevelTime,
    MenuState,
    HintState,
    TotalCompletion,
    VideoResource,
    DailyChallenges,
);
#[derive(MavericRoot)]
pub struct ViewRoot;

impl MavericRootChildren for ViewRoot {
    type Context = ViewContext;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child("Top Bar", TopBar, context);

        if context.5.is_closed() {
            let level_complete = context.2.is_level_complete();

            commands.add_child("cells", GridTiles { level_complete }, context);
            commands.add_child("words", WordsNode, context);

            match context.1.level(&context.9) {
                itertools::Either::Left(level) => {
                    let close_to_solution =
                        context.0.is_close_to_a_solution(level, context.2.as_ref());
                    commands.add_child(
                        "word_line",
                        WordLine {
                            solution: context.0.solution.clone(),
                            should_hide: context.0.is_just_finished,
                            close_to_solution,
                        },
                        &context.3,
                    );

                    if context.2.is_level_complete() {
                        commands.add_child("congrats", CongratsView, context);
                    }

                    if let Some(text) = TutorialText::try_create(&context.1, &context.2) {
                        commands.add_child("tutorial", TutorialNode { text }, &context.3);
                    } else {
                        let theme = level.name.clone();
                        commands.add_child("ui_theme", LevelName { theme }, &context.3);

                        if let Some(info) = &level.extra_info{
                            commands.add_child("ui_theme_info", LevelExtraInfo { info: info.clone() }, &context.3);
                        }

                        let total_seconds = context.4.as_ref().total_elapsed().as_secs();
                        let time_text = format_seconds(total_seconds);
                        commands.add_child("ui_timer", UITimer { time_text }, &context.3);

                        if let Some(playness) = match context.4.as_ref(){
                            LevelTime::Running { .. } => Some(0.0), //if running show pause
                            LevelTime::Paused { .. } => Some(1.0), // if paused show running
                            LevelTime::Finished { .. } => None,
                        }{

                            const SDF_PARAMETERS: &[ShaderParameter] = &[ShaderParameter::f32(0)];
                            let timer_rect = context.3.get_rect(&GameLayoutEntity::Timer, &());
                            commands.add_child("timer_play_pause",
                            SmudShapeNode{
                               color: Color::BLACK,
                               sdf: PLAY_PAUSE_SHADER_PATH,
                               fill: SIMPLE_FILL_SHADER_PATH,
                               frame_size: 1.0,
                               f_params:[playness, 0.0, 0.0, 0.0, 0.0, 0.0],
                               u_params:[0;SHAPE_U_PARAMS],
                               fill_param_usage: ShaderParamUsage::NO_PARAMS,
                               sdf_param_usage: ShaderParamUsage(SDF_PARAMETERS)
                            }.with_bundle(Transform{
                                translation: (timer_rect.centre_right() - Vec2{x: timer_rect.height(), y: 0.0}).extend(crate::z_indices::TIMER),
                                rotation: Default::default(),
                                scale: Vec3::ONE * timer_rect.height()
                            })
                            .with_transition_to::<SmudParamLens<0>>(playness, 2.0.into())
                            , &())
                        }


                    }
                }
                itertools::Either::Right(non_level) => {
                    commands.add_child("non_level", NonLevelView { non_level }, &context.3);
                }
            }
        } else {
            commands.add_child("menu", Menu, context);
        }
    }
}
