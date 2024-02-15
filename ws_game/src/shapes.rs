use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_param_shaders::prelude::*;
use bytemuck::Pod;
use bytemuck::Zeroable;
use maveric::node::MavericNode;
use maveric::prelude::*;
use std::fmt::Debug;

use crate::prelude::ButtonInteraction;
use crate::prelude::PressedButton;
use crate::startup::ADDITIONAL_TRACKING;

maveric::define_lens!(RoundingLens, ShaderRounding, f32, rounding);
maveric::define_lens!(ProgressLens, ShaderProgress, f32, progress);
maveric::define_lens!(ShaderColorLens, ShaderColor, Color, color);

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractToShaderPlugin::<BasicBoxShaderExtraction>::default());
        app.add_plugins(ExtractToShaderPlugin::<ButtonBoxShaderExtraction>::default());
        app.add_plugins(ExtractToShaderPlugin::<BoxWithBorderShader>::default());
        //app.add_plugins(ExtractToShaderPlugin::<CircleShader>::default());
        app.add_plugins(ExtractToShaderPlugin::<SparkleShader>::default());
        app.add_plugins(ExtractToShaderPlugin::<
            crate::prelude::fireworks::FireworksShader,
        >::default());

        app.register_transition::<ProgressLens>();
        app.register_transition::<RoundingLens>();
        app.register_transition::<ShaderColorLens>();
        app.register_transition::<TextColorLens<0>>();
    }
}

#[derive(Debug, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "a31d800c-02a2-4db7-8aaf-1caa2bd1dc37"]
pub struct BoxShader;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct ButtonBoxShaderExtraction;

impl ExtractToShader for ButtonBoxShaderExtraction {
    type Shader = BoxWithBorderShader;
    type ParamsQuery<'a> = (
        &'a ShaderColor,
        &'a ShaderRounding,
        &'a ShaderProportions,
        &'a ShaderSecondColor,
        &'a ShaderBorder,
        &'a ButtonInteraction,
    );
    type ParamsBundle = (
        ShaderColor,
        ShaderRounding,
        ShaderProportions,
        ShaderSecondColor,
        ShaderBorder,
        ButtonInteraction, //todo make interaction optional
    );
    type ResourceParams<'w> = (Res<'w, PressedButton>, Res<'w, Time>);

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        let (color, rounding, proportions, color2, shader_border, button_interaction) = query_item;

        let (pressed_button, time) = resource;

        if let Some(duration) = match pressed_button.as_ref() {
            PressedButton::None | PressedButton::NoInteractionPressed { .. } => None,
            PressedButton::Pressed {
                interaction,
                start_elapsed,
                ..
            } => {
                if interaction == button_interaction {
                    Some(time.elapsed().saturating_sub(*start_elapsed))
                } else {
                    None
                }
            }
            PressedButton::PressedAfterActivated { .. } => None,
        } {
            const TRANSITION_SECS: f32 = 0.2;
            let ratio = (duration.as_secs_f32() / TRANSITION_SECS).clamp(0.0, 1.0);

            let rgba1: LinearRGBA = color.color.into();
            let rgba2: LinearRGBA = color2.color.into();

            let new_color = rgba2 * ratio + (rgba1 * (1.0 - ratio));

            BoxWithBorderShaderParams {
                color: new_color,
                rounding: rounding.rounding,
                width: proportions.width,
                height: proportions.height,
                border_color: shader_border.border_color.into(),
                border: shader_border.border,
            }
        } else {
            BoxWithBorderShaderParams {
                color: color.color.into(),
                rounding: rounding.rounding,
                width: proportions.width,
                height: proportions.height,
                border_color: shader_border.border_color.into(),
                border: shader_border.border,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BasicBoxShaderExtraction;

impl ExtractToShader for BasicBoxShaderExtraction {
    type Shader = BoxShader;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderRounding, &'a ShaderProportions);
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderProportions);
    type ResourceParams<'w> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        BoxShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,
            height: query_item.2.height,
            width: query_item.2.width,
        }
    }
}

impl ParameterizedShader for BoxShader {
    type Params = BoxShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.width, in.height, in.rounding)",
            fill_color: "fill::simple::fill(d, in.color, in.pos)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [SIMPLE_FILL_IMPORT, BOX_SDF_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct BoxShaderParams {
    pub color: LinearRGBA,
    // Width as a proportion of scale in range 0..=1.0
    pub width: f32,

    // Height as a proportion of scale in range 0..=1.0
    pub height: f32,
    pub rounding: f32,
}

impl ShaderParams for BoxShaderParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct HorizontalGradientBoxShaderParams {
    pub color: LinearRGBA,
    /// height as a proportion of scale
    pub height: f32,
    /// width as a proportion of scale
    pub width: f32,
    pub rounding: f32,
    pub progress: f32,
    pub color2: LinearRGB,
}

impl ShaderParams for HorizontalGradientBoxShaderParams {}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderColor {
    pub color: Color,
}

impl From<Color> for ShaderColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderSecondColor {
    pub color: Color,
}

impl From<Color> for ShaderSecondColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct ShaderRounding {
    pub rounding: f32,
}

impl From<f32> for ShaderRounding {
    fn from(rounding: f32) -> Self {
        Self { rounding }
    }
}
impl Default for ShaderRounding {
    fn default() -> Self {
        Self { rounding: 0.0 }
    }
}

/// height / width
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct ShaderProportions {
    /// width in range 0..=1.0
    pub width: f32,
    /// height in range 0..=1.0
    pub height: f32,
}

impl Default for ShaderProportions {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
        }
    }
}

impl From<Vec2> for ShaderProportions {
    fn from(value: Vec2) -> Self {
        Self {
            width: value.x,
            height: value.y,
        }
    }
}

#[derive(Debug, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "df3562db-60d2-471a-81ac-616fb633c7e7"]
pub struct BoxWithBorderShader;

impl ExtractToShader for BoxWithBorderShader {
    type Shader = Self;

    type ParamsQuery<'a> = (
        &'a ShaderColor,
        &'a ShaderRounding,
        &'a ShaderProportions,
        &'a ShaderBorder,
    );
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderProportions, ShaderBorder);
    type ResourceParams<'w> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        BoxWithBorderShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,

            width: query_item.2.width,
            height: query_item.2.height,
            border_color: query_item.3.border_color.into(),
            border: query_item.3.border,
        }
    }
}

impl ParameterizedShader for BoxWithBorderShader {
    type Params = BoxWithBorderShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.width, in.height, in.rounding)",
            fill_color: "fill::fill_with_outline::fill(d, in.color, in.border, in.border_color)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [FILL_WITH_OUTLINE_IMPORT, BOX_SDF_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct BoxWithBorderShaderParams {
    pub width: f32,
    pub height: f32,
    pub rounding: f32,

    pub color: LinearRGBA,
    pub border_color: LinearRGBA,
    pub border: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderBorder {
    pub border_color: Color,
    pub border: f32,
}

impl ShaderBorder {
    pub const NONE: Self = ShaderBorder {
        border_color: Color::NONE,
        border: 0.0,
    };

    pub fn from_color(color: Color) -> Self {
        Self {
            border_color: color,
            border: 0.01,
        }
    }
}

impl ShaderParams for BoxWithBorderShaderParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable, Component)]
pub struct ShaderProgress {
    pub progress: f32,
}

impl ShaderParams for ShaderProgress {}

impl From<f32> for ShaderProgress {
    fn from(progress: f32) -> Self {
        Self { progress }
    }
}

// #[derive(Debug, TypeUuid, Default, PartialEq, Clone, Copy)]
// #[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
// pub struct CircleShader;

// impl ExtractToShader for CircleShader {
//     type Shader = Self;
//     type ParamsQuery<'a> = &'a ShaderColor;
//     type ParamsBundle = ShaderColor;
//     type ResourceParams<'w> = ();

//     fn get_params(
//         query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
//         _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
//     ) -> <Self::Shader as ParameterizedShader>::Params {
//         ColorParams {
//             color: query_item.color.into(),
//         }
//     }
// }

// impl ParameterizedShader for CircleShader {
//     type Params = ColorParams;

//     fn fragment_body() -> impl Into<String> {
//         SDFColorCall {
//             sdf: "sdf::circle::sdf(in.pos)",
//             fill_color: "fill::simple::fill(d, in.color, in.pos)",
//         }
//     }

//     fn imports() -> impl Iterator<Item = FragmentImport> {
//         [SIMPLE_FILL_IMPORT, CIRCLE_IMPORT].into_iter()
//     }

//     const FRAME: Frame = Frame::square(1.0);
// }

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams {}

#[derive(Debug, TypeUuid, Default, PartialEq, Clone, Copy)]
#[uuid = "a105f872-0a73-4226-a9ee-92518c947847"]
pub struct SparkleShader;

impl ExtractToShader for SparkleShader {
    type Shader = Self;
    type ParamsQuery<'a> = &'a SparkleParams;
    type ParamsBundle = SparkleParams;
    type ResourceParams<'w> = Res<'w, Time>;

    fn get_params(
        q: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        r: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        SparkleShaderParams {
            count1: q.count1,
            count2: q.count2,
            seed: q.seed,
            time: r.elapsed_seconds_wrapped(),
        }
    }
}

impl ParameterizedShader for SparkleShader {
    type Params = SparkleShaderParams;

    fn fragment_body() -> impl Into<String> {
        "return fill::sparkle::fill(in.pos, in.count1, in.count2, in.seed, in.time);"
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [SPARKLE_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SparkleShaderParams {
    pub count1: f32,
    pub count2: f32,
    pub seed: f32,
    pub time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Component)]
pub struct SparkleParams {
    pub count1: f32,
    pub count2: f32,
    pub seed: f32,
}

impl ShaderParams for SparkleShaderParams {}

pub(crate) const SIMPLE_FILL_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/simple.wgsl",
    import_path: "fill::simple",
};

pub const HORIZONTAL_GRADIENT_FILL: FragmentImport = FragmentImport {
    //TODO rename
    path: "shaders/fill/horizontal_gradient.wgsl",
    import_path: "fill::horizontal_gradient",
};

pub const BOX_SDF_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/sdf/box.wgsl",
    import_path: "shaders::box",
};

const FILL_WITH_OUTLINE_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/fill_with_outline.wgsl",
    import_path: "fill::fill_with_outline",
};

const SPARKLE_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/sparkle.wgsl",
    import_path: "fill::sparkle",
};

// const CIRCLE_IMPORT: FragmentImport = FragmentImport {
//     path: "shaders/sdf/circle.wgsl",
//     import_path: "sdf::circle",
// };

pub fn basic_box_node1(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,
    rounding: f32,
) -> impl MavericNode<Context = ()> {
    let height = height.abs();
    let width = width.abs();
    let scale = height.max(width);

    ShaderBundle::<BasicBoxShaderExtraction> {
        parameters: (
            color.into(),
            rounding.into(),
            ShaderProportions {
                width: width / scale,
                height: height / scale,
            },
        ),
        transform: Transform {
            translation,
            scale: Vec3::ONE * scale * 0.5,
            ..Default::default()
        },
        ..default()
    }
}

pub fn button_box_node(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,
    color2: Color,

    rounding: f32,
    border: ShaderBorder,
    button_interaction: ButtonInteraction,
) -> impl MavericNode<Context = ()> {
    let height = height.abs();
    let width = width.abs();
    let scale = height.max(width);

    ShaderBundle::<ButtonBoxShaderExtraction> {
        parameters: (
            color.into(),
            rounding.into(),
            ShaderProportions {
                width: width / scale,
                height: height / scale,
            },
            color2.into(),
            border,
            button_interaction,
        ),
        transform: Transform {
            translation,
            scale: Vec3::ONE * scale * 0.5,
            ..Default::default()
        },
        ..default()
    }
}

pub fn box_with_border_node(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,

    rounding: f32,
    border: ShaderBorder,
) -> impl MavericNode<Context = ()> {
    let height = height.abs();
    let width = width.abs();
    let scale = height.max(width);

    ShaderBundle::<BoxWithBorderShader> {
        parameters: (
            color.into(),
            rounding.into(),
            ShaderProportions {
                width: width / scale,
                height: height / scale,
            },
            border,
        ),
        transform: Transform {
            translation,
            scale: Vec3::ONE * scale * 0.5,
            ..Default::default()
        },
        ..default()
    }
}
