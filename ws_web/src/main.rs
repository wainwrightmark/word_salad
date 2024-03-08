pub mod ads;
pub mod purchases;

use ads::AdsPlugin;
use purchases::PurchasesPlugin;
use ws_common::prelude::*;
use ws_core::layout::entities::IDEAL_WIDTH;
use ws_core::layout::entities::IDEAL_HEIGHT;



#[cfg(feature="recording")]
use bevy::{core::FrameCount, core_pipeline::bloom::BloomSettings, prelude::*, render::{camera::RenderTarget, render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages}}};
#[cfg(feature="recording")]
use bevy_image_export::{ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource};

fn main() {
    ws_common::startup::setup_app(add_web_plugins);
}

fn add_web_plugins(app: &mut App) {
    app.add_plugins(PurchasesPlugin);
    app.add_plugins(AdsPlugin);

    #[cfg(feature="recording")]
    {
        app.add_plugins(ImageExportPlugin::default());
        app.add_systems(Startup, setup_recording);
    }
}

#[cfg(feature="recording")]
fn setup_recording(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
) {

    // Create an output texture.


    let output_texture_handle = {
        let size = Extent3d {
            width: IDEAL_WIDTH as u32,
            height: IDEAL_HEIGHT as u32,
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
            camera: Camera{
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