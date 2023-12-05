use std::fmt::Debug;

use bevy::prelude::*;
use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
use bevy_smud::{Frame, SmudPlugin, SmudShaders, SmudShape};
use maveric::node::MavericNode;
use maveric::prelude::*;
use maveric::with_bundle::CanWithBundle;

pub const SHAPE_F_PARAMS: usize = 6;
pub const SHAPE_U_PARAMS: usize = 4;

pub type SmudShapeWithParams = SmudShape<SHAPE_F_PARAMS, SHAPE_U_PARAMS>;
pub type SmudShapeFParams = [f32; SHAPE_F_PARAMS];
pub type SmudShapeUParams = [u32; SHAPE_U_PARAMS];

maveric::define_lens!(SmudColorLens, SmudShapeWithParams, Color, color);
maveric::define_lens!(
    SmudParamsLens,
    SmudShapeWithParams,
    SmudShapeFParams,
    f_params
);

pub type SmudParamLens<const INDEX: usize> =
    Prism2<SmudParamsLens, ElementAtLens<INDEX, SHAPE_F_PARAMS, f32>>;

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SmudPlugin::<SHAPE_F_PARAMS, SHAPE_U_PARAMS>);

        app.register_transition::<SmudColorLens>();
        app.register_transition::<SmudParamLens<0>>();
        app.register_transition::<SmudParamLens<1>>();
        app.register_transition::<SmudParamLens<2>>();
        app.register_transition::<SmudParamLens<3>>();
        app.register_transition::<SmudParamLens<4>>();

        app.add_systems(PostStartup, preload_shaders);
    }
}

#[derive(Debug, PartialEq)]
pub struct SmudShapeNode {
    pub color: Color,
    pub sfd: &'static str,
    pub fill: &'static str,
    pub frame_size: f32,
    pub f_params: [f32; SHAPE_F_PARAMS],
    pub u_params: [u32; SHAPE_U_PARAMS],
    pub sdf_param_usage: ShaderParamUsage,
    pub fill_param_usage: ShaderParamUsage,
}

impl MavericNode for SmudShapeNode {
    type Context = ();

    fn set_components(mut commands: maveric::prelude::SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle((VisibilityBundle::default(), GlobalTransform::default()));

        commands.advanced(|args, c| {
            if !args.is_hot() {
                return;
            }
            let asset_server = c
                .get_res_untracked::<AssetServer>()
                .expect("SmudShapeNode should be able to get asset server");

            let node = args.node;
            let sdf = asset_server.load(node.sfd);
            let fill = asset_server.load(node.fill);

            let shaders = SmudShaders::<SHAPE_F_PARAMS, SHAPE_U_PARAMS> {
                sdf,
                fill,
                sdf_param_usage: node.sdf_param_usage,
                fill_param_usage: node.fill_param_usage,
            };

            let shape = SmudShape::<SHAPE_F_PARAMS, SHAPE_U_PARAMS> {
                color: node.color,

                frame: Frame::Quad(node.frame_size),
                f_params: node.f_params,
                u_params: node.u_params,
            };
            c.insert(shaders);
            c.insert(shape);
        });
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.no_children()
    }
}

fn preload_shaders(asset_server: Res<AssetServer>) {
    //force all shaders to stay loaded
    for shader in [
        BOX_SHADER_PATH,
        BOX_BORDER_SHADER_PATH,
        WORD_LINE_SHADER_PATH,
        SIMPLE_FILL_SHADER_PATH,
        CIRCLE_SHADER_PATH,
        WORD_LINE_FILL_SHADER_PATH,
        FILL_WITH_OUTLINE_SHADER_PATH,
        ANYWHERE_SHADER_PATH,
        SPARKLE_SHADER_PATH,
        VORONOI_SHADER_PATH,
        GRADIENT_SHADER_PATH,
        HORIZONTAL_GRADIENT_SHADER_PATH,
    ] {
        let handle: Handle<Shader> = asset_server.load(shader);
        match handle {
            Handle::Strong(s) => {
                let b = Box::new(s);
                Box::leak(b); //this is so ugly but it works :)
            }
            Handle::Weak(_w) => {
                warn!("Preloaded asset was weak")
            }
        }
    }
}

pub const BOX_SHADER_PATH: &str = "shaders/sdf/box.wgsl";
pub const BOX_BORDER_SHADER_PATH: &str = "shaders/sdf/box_border.wgsl";
pub const WORD_LINE_SHADER_PATH: &str = "shaders/sdf/word_line.wgsl";
pub const ANYWHERE_SHADER_PATH: &str = "shaders/sdf/anywhere.wgsl";
pub const CIRCLE_SHADER_PATH: &str = "shaders/sdf/circle.wgsl";

pub const SIMPLE_FILL_SHADER_PATH: &str = "shaders/fill/simple.wgsl";
pub const FILL_WITH_OUTLINE_SHADER_PATH: &str = "shaders/fill/fill_with_outline.wgsl";
pub const WORD_LINE_FILL_SHADER_PATH: &str = "shaders/fill/word_line_fill.wgsl";
pub const SPARKLE_SHADER_PATH: &str = "shaders/fill/sparkle.wgsl";
pub const VORONOI_SHADER_PATH: &str = "shaders/fill/voronoi_gradient.wgsl";
pub const GRADIENT_SHADER_PATH: &str = "shaders/fill/gradient.wgsl";
pub const HORIZONTAL_GRADIENT_SHADER_PATH: &str = "shaders/fill/horizontal_gradient.wgsl";

pub fn box_node(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,
    rounding: f32,
) -> impl MavericNode<Context = ()> {
    const SDF_PARAMETERS: &[ShaderParameter] =
        &[ShaderParameter::f32(0), ShaderParameter::f32(1)];

    let scale = width;
    SmudShapeNode {
        color,
        sfd: BOX_SHADER_PATH,
        fill: SIMPLE_FILL_SHADER_PATH,
        frame_size: (1.0f32).max(height / scale),
        f_params: [
            (height / scale),
            rounding,
            rounding,
            0.0,
            0.0,
            0.0,
        ],
        u_params: Default::default(),
        sdf_param_usage: ShaderParamUsage(SDF_PARAMETERS),
        fill_param_usage: ShaderParamUsage::NO_PARAMS,
    }
    .with_bundle(Transform {
        translation,
        scale: Vec3::ONE * scale * 0.5,
        ..Default::default()
    })
}

pub fn box_border_node(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,
    rounding: f32,
    border_proportion: f32,
) -> impl MavericNode<Context = ()> {
    const PARAMETERS: &[ShaderParameter] = &[
        ShaderParameter::f32(0),
        ShaderParameter::f32(1),
        ShaderParameter::f32(2),
        ShaderParameter::f32(3),
    ];

    let scale = width.max(height);
    SmudShapeNode {
        color,
        sfd: BOX_BORDER_SHADER_PATH,
        fill: SIMPLE_FILL_SHADER_PATH,
        frame_size: 1.0,

        f_params: [
            (width / scale),
            (height / scale),
            rounding,
            border_proportion,
            0.0,
            0.0,
        ],
        u_params: Default::default(),
        sdf_param_usage: ShaderParamUsage(PARAMETERS),
        fill_param_usage: ShaderParamUsage::NO_PARAMS,
    }
    .with_bundle(Transform {
        translation,
        scale: Vec3::ONE * scale * 0.5,
        ..Default::default()
    })
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
    const SDF_PARAMETERS: &[ShaderParameter] =
        &[ShaderParameter::f32(0), ShaderParameter::f32(1)];

    const FILL_PARAMETERS: &[ShaderParameter] = &[
        ShaderParameter::f32(2),
        ShaderParameter::f32(3),
        ShaderParameter::f32(4),
        ShaderParameter::f32(5),
    ];

    let scale = width;
    SmudShapeNode {
        color,
        sfd: BOX_SHADER_PATH,
        fill: FILL_WITH_OUTLINE_SHADER_PATH,
        frame_size: (1.0f32).max(height / scale),

        f_params: [
            (height / scale),
            rounding,
            border_proportion,
            border_color.r(),
            border_color.g(),
            border_color.b(),
        ],
        u_params: Default::default(),
        sdf_param_usage: ShaderParamUsage(SDF_PARAMETERS),
        fill_param_usage: ShaderParamUsage(FILL_PARAMETERS),
    }
    .with_bundle(Transform {
        translation,
        scale: Vec3::ONE * scale * 0.5,
        ..Default::default()
    })
}
