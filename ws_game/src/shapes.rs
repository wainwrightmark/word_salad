use bevy::prelude::*;
use bevy_smud::{Frame, SmudPlugin, SmudShape};
use maveric::node::MavericNode;
use maveric::prelude::*;

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SmudPlugin);

        app.register_transition::<SmudColorLens>();
    }
}

maveric::define_lens!(SmudColorLens, SmudShape, Color, color);

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
