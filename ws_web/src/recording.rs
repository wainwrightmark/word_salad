use std::{collections::VecDeque, time::Duration};

use itertools::Either;
use ws_common::{prelude::*, startup::ADDITIONAL_TRACKING};

use bevy::prelude::*;

use bevy::render::{
    camera::RenderTarget,
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};

use bevy_image_export::{
    ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource,
};

pub struct RecordingPlugin;

impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ImageExportPlugin::default());
        app.add_systems(Startup, setup_recording);
        app.add_systems(PreUpdate, handle_keyboard_events);
        app.add_systems(PreUpdate, handle_timed_events);
        app.add_systems(PostStartup, change_level_on_start);
    }
}

struct TimedEvents {
    events: VecDeque<f32>,
}

impl Default for TimedEvents {
    fn default() -> Self {
        Self {
            events: VecDeque::from([
                // 14.435799, 14.776774, 14.983564, 15.197325, 15.580590, 16.026654, 16.393288,
                // 16.619239, 16.826251, 17.229206, 17.635556, 18.041905, 18.448254, 18.866213,
                // 19.284172, 19.713741, 20.178141, 21.118549, 21.401758, 21.629388, 21.822354,
                // 22.236440, 22.689894, 23.025056, 23.163064, 23.429222, 23.826693, 24.272269,
                // 24.671202, 25.158821,
                // 25.669660,


                0.011610, 0.812698, 1.567347, 1.938866, 2.310385, 2.728345,

                            3.134694, 3.541043,
                           3.924172, 4.307302, 4.713651, 5.120000, 5.526349, 5.944308,

                            6.362268, 6.745397,
                           7.140136, 7.500045, 7.871565, 8.243084, 8.637823, 9.020952,

                            9.415692, 9.798821,
                           10.193560, 10.599909, 11.006259, 11.424218, 11.842177, 12.248526, 12.666485,
                           13.084444, 13.490794,
            ]),
        }
    }
}

fn change_level_on_start(
    mut current_level: ResMut<CurrentLevel>,
    mut found_words_state: ResMut<FoundWordsState>,
    mut level_time: ResMut<LevelTime>,
) {
    let level = DesignedLevel::from_tsv_line(
        //"SPCHPALITEMOSUNA	#2 Fruits[by samuel]	Apple	Lemon	Lime	Melon	Peach	Satsuma",
        "AKLSRASAHBNIODED	#1 US States[by mark]	Alaska	Arkansas	Kansas	Nebraska	Rhode Island",
    )
    .unwrap();
    *found_words_state = FoundWordsState::new_from_level(&level);
    let custom_level = CurrentLevel::Custom {
        name: level.full_name().clone(),
    };

    info!("Changing to custom level");
    set_custom_level(level);

    *current_level = custom_level;
    *level_time = LevelTime::Running {
        since: chrono::Utc::now(),
        additional: Duration::ZERO,
    }
}


fn handle_timed_events(
    mut chosen_state: ResMut<ChosenState>,
    found_words_state: Res<FoundWordsState>,
    current_level: Res<CurrentLevel>,
    daily_challenges: Res<DailyChallenges>,
    mut input_state: Local<GridInputState>,
    mut timed_events: Local<TimedEvents>,
    time: Res<Time>,
) {
    const OFFSET: f32 = 2.0;
    if let Some(peeked) = timed_events.events.front() {
        if time.elapsed_seconds() < (*peeked) + OFFSET {
            return;
        }
        info!(
            "Doing event {:2.3} at {:2.3}: {:2.3}",
            peeked,
            time.elapsed_seconds(),
            (time.elapsed_seconds() - (*peeked + OFFSET))
        );
        timed_events.events.pop_front();

        do_next_event(
            &current_level,
            &daily_challenges,
            &found_words_state,
            &mut chosen_state,
            &mut input_state,
        );
    }
    ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

fn handle_keyboard_events(
    mut chosen_state: ResMut<ChosenState>,
    found_words_state: Res<FoundWordsState>,
    current_level: Res<CurrentLevel>,
    daily_challenges: Res<DailyChallenges>,
    keyboard_events: Res<ButtonInput<KeyCode>>,
    mut input_state: Local<GridInputState>,
) {
    if keyboard_events.just_pressed(KeyCode::Space) {
        do_next_event(
            &current_level,
            &daily_challenges,
            &found_words_state,
            &mut chosen_state,
            &mut input_state,
        );
    }
}

fn do_next_event(
    current_level: &CurrentLevel,
    daily_challenges: &DailyChallenges,
    found_words_state: &FoundWordsState,
    chosen_state: &mut ResMut<ChosenState>,
    input_state: &mut Local<GridInputState>,
) {
    if let Either::Left(level) = current_level.level(&daily_challenges) {
        const SOLUTION_ORDER: &[usize] = &[2,3,1,4,0];

        if let Some(so) = SOLUTION_ORDER
            .into_iter()
            .filter(|so| {
                found_words_state
                    .word_completions
                    .get(**so)
                    .is_some_and(|c| !c.is_complete())
            })
            .next()
        {
            let word = level.words.get(*so).unwrap();
            if let Some(solution) =
                word.find_solution_with_tiles(&level.grid, found_words_state.unneeded_tiles)
            {
                if let Some(tile) = solution.get(chosen_state.current_solution().len()) {
                    info!("Selecting {tile} in '{}'", word.text);
                    input_state.handle_input_start(
                        chosen_state,
                        *tile,
                        &level.grid,
                        &found_words_state,
                    );

                    input_state.handle_input_end_no_location();
                    ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        } else {
            warn!("No more words in solution order");
        }
    } else {
        warn!("No level");
    }
}

fn setup_recording(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
) {
    // Create an output texture.

    let output_texture_handle = {
        let size = Extent3d {
            width: DEFAULT_WINDOW_WIDTH as u32,
            height: DEFAULT_WINDOW_HEIGHT as u32,
            ..default()
        };
        let mut export_texture = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        export_texture.resize(size);

        images.add(export_texture)
    };

    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_translation(1000.0 * Vec3::Z),
            camera: Camera {
                order: 2,
                ..Default::default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(Camera2dBundle {
                camera: Camera {
                    // Connect the output texture to a camera as a RenderTarget.
                    target: RenderTarget::Image(output_texture_handle.clone()),
                    ..default()
                },
                ..default()
            });
        });

    // Spawn the ImageExportBundle to initiate the export of the output texture.
    commands.spawn(ImageExportBundle {
        source: export_sources.add(output_texture_handle),
        settings: ImageExportSettings {
            // Frames will be saved to "./out/[#####].png".
            output_dir: "out".into(),
            // Choose "exr" for HDR renders.
            extension: "png".into(),
        },
    });
}