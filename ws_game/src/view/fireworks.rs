use crate::{prelude::*, z_indices};
use bevy::reflect::TypeUuid;
use bevy_param_shaders::prelude::*;

use num_traits::Zero;
use rand::Rng;
use std::time::Duration;

use ws_core::layout::entities::*;

pub fn create_firework(
    cb: &mut ChildBuilder,
    rng: &mut impl Rng,

    total_seconds: f32,

    size: &Size,
    delay_sec: f32,
    start_straight_after_delay: bool,

    selfie_mode: SelfieMode,
) {
    let rect = size.get_rect(
        &ws_core::layout::entities::GameLayoutEntity::Grid,
        &selfie_mode,
    );

    let delay = delay_sec
        + if start_straight_after_delay {
            0.0
        } else {
            rng.gen_range(0.0..(total_seconds - 1.0))
        };

    let color = if selfie_mode.is_selfie_mode {
        Color::hsl(rng.gen_range(0.0..=360.0), 1.0, 0.75)
    } else {
        let hue = rng.gen_range(200.0..=440.0) % 360.0;
        Color::hsl(hue, 1.0, 0.75)
    };

    let position = rect.top_left
        + Vec2 {
            x: rng.gen_range(0.0..=(rect.extents.x.abs())),
            y: rng.gen_range(rect.extents.y..=0.0),
        };

    let bundle = (
        ShaderBundle::<FireworksShader> {
            parameters: (color.into(), 0.0.into()),
            transform: Transform {
                translation: position.extend(z_indices::FIREWORKS),

                scale: Vec3::ONE * rect.extents.x.max(rect.extents.y),
                ..default()
            },
            ..default()
        },
        ScheduledForDeletion {
            remaining: Duration::from_secs_f32(1.0),
        },
        TransitionBuilder::<ProgressLens>::default()
            .then_tween(1.0, 1.0.into())
            .build(),
    );

    if delay.is_zero() {
        cb.spawn(bundle);
    } else {
        cb.spawn(ScheduledChange {
            remaining: Duration::from_secs_f32(delay),
            boxed_change: Box::new(move |ec| {
                ec.insert(bundle);
            }),
        });
    }
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "3b76cd37-5f82-4fcc-9d26-ea6f60d616e3"]
pub struct FireworksShader;

impl ExtractToShader for FireworksShader {
    type Shader = Self;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderProgress);
    type ParamsBundle = (ShaderColor, ShaderProgress);
    type ResourceParams<'w> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        FireworksParams {
            color: query_item.0.color.into(),
            progress: query_item.1.progress,
        }
    }
}

impl ParameterizedShader for FireworksShader {
    type Params = FireworksParams;

    fn fragment_body() -> impl Into<String> {
        "return fill::fireworks::fill(in.color, in.pos, in.progress);"
    }

    fn imports() -> impl Iterator<Item = bevy_param_shaders::prelude::FragmentImport> {
        [FIREWORKS_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(0.25);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FireworksParams {
    pub color: LinearRGB,
    pub progress: f32,
}

impl ShaderParams for FireworksParams {}

const FIREWORKS_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/fireworks.wgsl",
    import_path: "fill::fireworks",
};
