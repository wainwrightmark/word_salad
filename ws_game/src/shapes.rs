use bevy::prelude::*;
use bevy_smud::{Frame, SmudPlugin, SmudShape};
use maveric::node::MavericNode;
use maveric::prelude::*;
use maveric::with_bundle::CanWithBundle;

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SmudPlugin);

        app.register_transition::<SmudColorLens>();

        app.add_systems(PostStartup, preload_shaders);
    }
}

maveric::define_lens!(SmudColorLens, SmudShape, Color, color);

fn preload_shaders(asset_server: Res<AssetServer>){
    //force all shaders to stay loaded
    for shader in [BOX_SHADER_PATH, BOX_BORDER_SHADER_PATH, SIMPLE_FILL_SHADER_PATH]{
        let handle: Handle<Shader> =  asset_server.load(shader);
        match handle{
            Handle::Strong(s) => {
                let b = Box::new(s);
                Box::leak(b); //this is so ugly but it works :)
            },
            Handle::Weak(_w) => {
                warn!("Preloaded asset was weak")
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SmudShapeNode {
    pub color: Color,
    pub sfd: &'static str,
    pub fill: &'static str,
    pub frame_size: f32,
    pub params: Vec4,
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

            let shape = SmudShape {
                color: node.color,
                sdf,
                fill,
                frame: Frame::Quad(node.frame_size),
                params: node.params,
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

fn box_params(width: f32, height: f32, rounding: f32) -> Vec4 {
    Vec4::new(width, height, rounding, 0.0)
}

fn box_border_params(width: f32, height: f32, rounding: f32, border_width: f32) -> Vec4 {
    Vec4::new(width, height, rounding, border_width)
}

pub const BOX_SHADER_PATH: &'static str = "shaders/sdf/box.wgsl";
pub const BOX_BORDER_SHADER_PATH: &'static str = "shaders/sdf/box_border.wgsl";

pub const SIMPLE_FILL_SHADER_PATH: &'static str = "shaders/fill/simple.wgsl";

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
        frame_size: 1.1,
        params: box_params(width / scale, height / scale, rounding),
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
        frame_size: 1.1,
        params: box_border_params(width / scale, height / scale, rounding, border_proportion),
    }
    .with_bundle(Transform {
        translation,
        scale: Vec3::ONE * scale * 0.5,
        ..Default::default()
    })
}
