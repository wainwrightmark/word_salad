use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_param_shaders::prelude::*;
use bytemuck::Pod;
use bytemuck::Zeroable;
use maveric::node::MavericNode;
use maveric::prelude::*;
use std::fmt::Debug;

use crate::prelude::FireworksShader;

maveric::define_lens!(RoundingLens, ShaderRounding, f32, rounding);
maveric::define_lens!(ProgressLens, ShaderProgress, f32, progress);
maveric::define_lens!(ShaderColorLens, ShaderColor, Color, color);
maveric::define_lens!(WordLineWidthLens, WordLineParams, f32, line_width);
maveric::define_lens!(WordLineProgressLens, WordLineParams, f32, progress);

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ParamShaderPlugin::<BoxShader>::default());
        app.add_plugins(ParamShaderPlugin::<HorizontalGradientBoxShader>::default());
        app.add_plugins(ParamShaderPlugin::<BoxWithBorderShader>::default());
        //app.add_plugins(ParamShaderPlugin::<PlayPauseShader>::default());
        app.add_plugins(ParamShaderPlugin::<CircleShader>::default());
        app.add_plugins(ParamShaderPlugin::<SparkleShader>::default());
        app.add_plugins(ParamShaderPlugin::<WordLineShader>::default());
        app.add_plugins(ParamShaderPlugin::<FireworksShader>::default());

        app.register_transition::<ProgressLens>();
        app.register_transition::<RoundingLens>();
        app.register_transition::<ShaderColorLens>();
        app.register_transition::<WordLineWidthLens>();
        app.register_transition::<WordLineProgressLens>();
        app.register_transition::<TextColorLens<0>>();
    }
}


#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "a31d800c-02a2-4db7-8aaf-1caa2bd1dc37"]
pub struct BoxShader;

impl ParameterizedShader for BoxShader {
    type Params = BoxShaderParams;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderRounding, &'a ShaderAspectRatio);
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderAspectRatio);

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.height, in.rounding)",
            fill_color: "fill::simple::fill(d, in.color, in.pos)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [SIMPLE_FILL_IMPORT, BOX_SDF_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        BoxShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,
            height: query_item.2.height,
        }
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct BoxShaderParams {
    pub color: LinearRGBA,
    // height as a proportion of width
    pub height: f32,
    pub rounding: f32,
}

impl ShaderParams for BoxShaderParams {}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "266b0619-b913-4cce-be86-7470ef0b129b"]
pub struct HorizontalGradientBoxShader;

impl ParameterizedShader for HorizontalGradientBoxShader {
    type Params = HorizontalGradientBoxShaderParams;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderRounding, &'a ShaderAspectRatio, &'a ShaderProgress, &'a ShaderSecondColor);
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderAspectRatio, ShaderProgress, ShaderSecondColor);

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.height, in.rounding)",
            fill_color: "fill::horizontal_gradient::fill(d, in.color, in.pos, in.progress, in.color2)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [HORIZONTAL_GRADIENT_FILL, BOX_SDF_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        HorizontalGradientBoxShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,
            height: query_item.2.height,
            progress: query_item.3.progress,
            color2: query_item.4.color.into()
        }
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct HorizontalGradientBoxShaderParams {
    pub color: LinearRGBA,
    // height as a proportion of width
    pub height: f32,
    pub rounding: f32,
    pub progress: f32,
    pub color2: LinearRGB
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

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderRounding {
    pub rounding: f32,
}

impl From<f32> for ShaderRounding {
    fn from(rounding: f32) -> Self {
        Self { rounding }
    }
}

/// height / width
#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderAspectRatio {
    pub height: f32,
}

impl From<f32> for ShaderAspectRatio {
    fn from(height: f32) -> Self {
        Self { height }
    }
}





#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "df3562db-60d2-471a-81ac-616fb633c7e7"]
pub struct BoxWithBorderShader;

impl ParameterizedShader for BoxWithBorderShader {
    type Params = BoxWithBorderShaderParams;
    type ParamsQuery<'a> = (
        &'a ShaderColor,
        &'a ShaderRounding,
        &'a ShaderAspectRatio,
        &'a ShaderBorder,
    );
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderAspectRatio, ShaderBorder);

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::box::sdf(in.pos, in.height, in.rounding)",
            fill_color: "fill::fill_with_outline::fill(d, in.color, in.border, in.border_color)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [FILL_WITH_OUTLINE_IMPORT, BOX_SDF_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        BoxWithBorderShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,
            height: query_item.2.height,
            border_color: query_item.3.border_color.into(),
            border: query_item.3.border,
        }
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct BoxWithBorderShaderParams {
    // height as a proportion of width
    pub height: f32,
    pub rounding: f32,

    pub color: LinearRGBA,
    pub border_color: LinearRGB,
    pub border: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderBorder {
    pub border_color: Color,
    pub border: f32,
}

impl ShaderParams for BoxWithBorderShaderParams {}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "67e11165-1873-42c2-9d6d-bfe7384eedc1"]
pub struct PlayPauseShader;

impl ParameterizedShader for PlayPauseShader {
    type Params = ShaderProgress;
    type ParamsQuery<'a> = &'a ShaderProgress;
    type ParamsBundle = ShaderProgress;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "shaders::play_pause::sdf(in.pos, in.progress)",
            fill_color: "fill::simple::fill(d, vec4<f32>(0.0,0.0,0.0,1.0), in.pos)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [SIMPLE_FILL_IMPORT, PLAY_PAUSE_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        *query_item
    }

    const FRAME: Frame = Frame::square(1.0);
}

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

#[repr(C)]
#[derive(Debug, TypeUuid, Default, PartialEq, Clone, Copy)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct CircleShader;

impl ParameterizedShader for CircleShader {
    type Params = ColorParams;
    type ParamsQuery<'a> = &'a ShaderColor;
    type ParamsBundle = ShaderColor;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "sdf::circle::sdf(in.pos)",
            fill_color: "fill::simple::fill(d, in.color, in.pos)",
        }
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [SIMPLE_FILL_IMPORT, CIRCLE_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        ColorParams {
            color: query_item.color.into(),
        }
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams {}

#[repr(C)]
#[derive(Debug, TypeUuid, Default, PartialEq, Clone, Copy)]
#[uuid = "a105f872-0a73-4226-a9ee-92518c947847"]
pub struct SparkleShader;

impl ParameterizedShader for SparkleShader {
    type Params = SparkleParams;
    type ParamsQuery<'a> = &'a SparkleParams;
    type ParamsBundle = SparkleParams;

    fn fragment_body() -> impl Into<String> {
        "return fill::sparkle::fill(in.pos, in.count1, in.count2, in.seed, globals.time);"
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [SPARKLE_IMPORT].into_iter()
    }

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        *query_item
    }

    const FRAME: Frame = Frame::square(1.0);

    const USE_TIME: bool = true; //TODO remove
}

#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component,
)]
pub struct SparkleParams {
    pub count1: f32,
    pub count2: f32,
    pub seed: f32,
}

impl ShaderParams for SparkleParams {}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "874f0666-0214-49b0-8e5a-b576fe098072"]
pub struct WordLineShader;

impl ParameterizedShader for WordLineShader {
    type Params = WordLineParams;
    type ParamsQuery<'a> = &'a WordLineParams;
    type ParamsBundle = WordLineParams;

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        *query_item
    }

    fn fragment_body() -> impl Into<String> {
        "return word_line::fill(in.pos, in.line_width, in.progress, in.points1, in.points2, in.points3, in.points4);"
    }

    fn imports() -> impl Iterator<Item = bevy_param_shaders::prelude::FragmentImport> {
        const WORDLINE_IMPORT: FragmentImport = FragmentImport {
            path: "shaders/fill/word_line.wgsl",
            import_path: "word_line",
        };

        [WORDLINE_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(1.0);
}

#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component,
)]
pub struct WordLineParams {
    pub line_width: f32,
    pub progress: f32,
    pub points1: u32,
    pub points2: u32,
    pub points3: u32,
    pub points4: u32,
}

impl ShaderParams for WordLineParams {}

// const CUBIC_FALLOFF_IMPORT: FragmentImport = FragmentImport {
//     path: "shaders/fill/cubic_falloff.wgsl",
//     import_path: "smud::default_fill",
// };



const SIMPLE_FILL_IMPORT: FragmentImport = FragmentImport{
    path: "shaders/fill/simple.wgsl",
    import_path: "fill::simple",
};

const HORIZONTAL_GRADIENT_FILL: FragmentImport = FragmentImport{
    path: "shaders/fill/horizontal_gradient.wgsl",
    import_path: "fill::horizontal_gradient",
};

const BOX_SDF_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/sdf/box.wgsl",
    import_path: "shaders::box",
};

const FILL_WITH_OUTLINE_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/fill_with_outline.wgsl",
    import_path: "fill::fill_with_outline",
};

const PLAY_PAUSE_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/sdf/play_pause.wgsl",
    import_path: "shaders::play_pause",
};

const SPARKLE_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/fill/sparkle.wgsl",
    import_path: "fill::sparkle",
};

const CIRCLE_IMPORT: FragmentImport = FragmentImport {
    path: "shaders/sdf/circle.wgsl",
    import_path: "sdf::circle",
};

pub fn box_node1(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,
    rounding: f32,
) -> impl MavericNode<Context = ()> {
    let scale = width;

    ShaderBundle {
        shape: ShaderShape::<BoxShader>::default(),
        parameters: (color.into(), rounding.into(), (height / scale).into()),
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
    border_color: Color,
    rounding: f32,
    border_proportion: f32,
) -> impl MavericNode<Context = ()> {
    let scale = width;
    ShaderBundle {
        shape: ShaderShape::<BoxWithBorderShader>::default(),
        parameters: (
            color.into(),
            rounding.into(),
            (height / scale).into(),
            ShaderBorder {
                border_color,
                border: border_proportion,
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
