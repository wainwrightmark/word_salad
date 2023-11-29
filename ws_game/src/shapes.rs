use std::fmt::Debug;

use bevy::prelude::*;
use bevy_smud::param_usage::ShaderParamUsage;
use bevy_smud::{Frame, SmudPlugin, SmudShape};
use maveric::node::MavericNode;
use maveric::prelude::*;
use maveric::with_bundle::CanWithBundle;

pub const SHAPE_PARAMS: usize = 8;

pub type SmudShapeWithParams = SmudShape<SHAPE_PARAMS>;
pub type SmudShapeParams = [f32; SHAPE_PARAMS];

maveric::define_lens!(SmudColorLens, SmudShapeWithParams, Color, color);
maveric::define_lens!(SmudParamsLens, SmudShapeWithParams, SmudShapeParams, params);

pub type SmudParamLens<const INDEX: usize> =
    Prism2<SmudParamsLens, ElementAtLens<INDEX, SHAPE_PARAMS, f32>>;

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SmudPlugin::<SHAPE_PARAMS>);

        app.register_transition::<SmudColorLens>();
        app.register_transition::<SmudParamLens<0>>();
        app.register_transition::<SmudParamLens<2>>();
        app.register_transition::<SmudParamLens<3>>();
        app.register_transition::<SmudParamLens<4>>();

        app.add_systems(PostStartup, preload_shaders);
    }
}

fn preload_shaders(asset_server: Res<AssetServer>) {
    //force all shaders to stay loaded
    for shader in [
        BOX_SHADER_PATH,
        BOX_BORDER_SHADER_PATH,
        WORD_LINE_SHADER_PATH,
        SIMPLE_FILL_SHADER_PATH,
        WORD_LINE_FILL_SHADER_PATH,
        FILL_WITH_OUTLINE_SHADER_PATH,
        ANYWHERE_SHADER_PATH,
        SPARKLE_SHADER_PATH,
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

#[derive(Debug, PartialEq)]
pub struct SmudShapeNode {
    pub color: Color,
    pub sfd: &'static str,
    pub fill: &'static str,
    pub frame_size: f32,
    pub params: [f32; 8],
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

            let shape = SmudShape::<SHAPE_PARAMS> {
                color: node.color,
                sdf,
                fill,
                frame: Frame::Quad(node.frame_size),
                params: node.params,
                sdf_param_usage: node.sdf_param_usage,
                fill_param_usage: node.fill_param_usage,
            };
            c.insert(shape);
        });
    }

    fn set_children<R: maveric::prelude::MavericRoot>(
        commands: maveric::prelude::SetChildrenCommands<Self, Self::Context, R>,
    ) {
        commands.no_children()
    }
}

// fn box_params(width: f32, height: f32, rounding: f32) -> Vec4 {
//     Vec4::new(width, height, rounding, 0.0)
// }

// fn box_border_params(width: f32, height: f32, rounding: f32, border_width: f32) -> Vec4 {
//     Vec4::new(width, height, rounding, border_width)
// }

pub const BOX_SHADER_PATH: &'static str = "shaders/sdf/box.wgsl";
pub const BOX_BORDER_SHADER_PATH: &'static str = "shaders/sdf/box_border.wgsl";
pub const WORD_LINE_SHADER_PATH: &'static str = "shaders/sdf/word_line.wgsl";
pub const ANYWHERE_SHADER_PATH: &'static str = "shaders/sdf/anywhere.wgsl";

pub const SIMPLE_FILL_SHADER_PATH: &'static str = "shaders/fill/simple.wgsl";
pub const FILL_WITH_OUTLINE_SHADER_PATH: &'static str = "shaders/fill/fill_with_outline.wgsl";
pub const WORD_LINE_FILL_SHADER_PATH: &'static str = "shaders/fill/word_line_fill.wgsl";
pub const SPARKLE_SHADER_PATH: &'static str = "shaders/fill/sparkle.wgsl";

pub fn box_node(
    width: f32,
    height: f32,
    translation: Vec3,
    color: Color,
    rounding: f32,
) -> impl MavericNode<Context = ()> {
    let scale = width.max(height);
    SmudShapeNode {
        color,
        sfd: BOX_SHADER_PATH,
        fill: SIMPLE_FILL_SHADER_PATH,
        frame_size: 1.0,
        params: [
            width / scale,
            height / scale,
            rounding,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ],
        sdf_param_usage: ShaderParamUsage::from_params(&[0, 1, 2]),
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
    let scale = width.max(height);
    SmudShapeNode {
        color,
        sfd: BOX_BORDER_SHADER_PATH,
        fill: SIMPLE_FILL_SHADER_PATH,
        frame_size: 1.0,

        params: [
            width / scale,
            height / scale,
            rounding,
            border_proportion,
            0.0,
            0.0,
            0.0,
            0.0,
        ],
        sdf_param_usage: ShaderParamUsage::from_params(&[0, 1, 2, 3]),
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
    let scale = width.max(height);
    SmudShapeNode {
        color,
        sfd: BOX_SHADER_PATH,
        fill: FILL_WITH_OUTLINE_SHADER_PATH,
        frame_size: 1.0,

        params: [
            width / scale,
            height / scale,
            rounding,
            0.0,
            border_proportion,
            border_color.r(),
            border_color.g(),
            border_color.b(),
        ],
        sdf_param_usage: ShaderParamUsage::from_params(&[0, 1, 2]),
        fill_param_usage: ShaderParamUsage::from_params(&[4, 5, 6, 7]),
    }
    .with_bundle(Transform {
        translation,
        scale: Vec3::ONE * scale * 0.5,
        ..Default::default()
    })
}
