pub mod ads;
pub mod purchases;
use ads::AdsPlugin;

use itertools::Either;
use purchases::PurchasesPlugin;
use ws_common::{prelude::*, startup::ADDITIONAL_TRACKING};

#[cfg(feature = "recording")]
use bevy::render::{
    camera::RenderTarget,
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
#[cfg(feature = "recording")]
use bevy_image_export::{
    ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource,
};

fn main() {
    ws_common::startup::setup_app(add_web_plugins);
}

fn add_web_plugins(app: &mut App) {
    app.add_plugins(PurchasesPlugin);
    app.add_plugins(AdsPlugin);

    #[cfg(feature = "recording")]
    {
        app.add_plugins(ImageExportPlugin::default());
        app.add_systems(Startup, setup_recording);
        app.add_systems(PreUpdate, handle_keyboard_events);
    }
}

#[cfg(feature = "recording")]
fn handle_keyboard_events(
    mut chosen_state: ResMut<ChosenState>,
    found_words_state: Res<FoundWordsState>,
    current_level: Res<CurrentLevel>,
    daily_challenges: Res<DailyChallenges>,
    keyboard_events: Res<ButtonInput<KeyCode>>,
    mut input_state: Local<GridInputState>,
) {
    if keyboard_events.just_pressed(KeyCode::Space) {
        if let Either::Left(level) = current_level.level(&daily_challenges) {
            const SOLUTION_ORDER: &[usize] = &[1, 2, 0, 3, 4];

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
                            &mut chosen_state,
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
}

#[cfg(feature = "recording")]
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
