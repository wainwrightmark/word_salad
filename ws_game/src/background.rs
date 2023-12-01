use bevy_smud::param_usage::ShaderParamUsage;
use maveric::{prelude::*, with_bundle::CanWithBundle};
use crate::prelude::*;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin{
    fn build(&self, app: &mut App) {
        app.register_maveric::<Background>();
    }
}

#[derive(Debug, Default, MavericRoot)]
pub struct Background;

impl MavericRootChildren for Background{
    type Context = MyWindowSize;

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        let scale= context.scaled_width.max(context.scaled_height);
        let node = SmudShapeNode {
            color: palette::BACKGROUND_COLOR_1.convert_color(),
            sfd: ANYWHERE_SHADER_PATH,
            fill: VORONOI_SHADER_PATH,
            frame_size: 1.0,
            params: [
                palette::BACKGROUND_COLOR_2.red,
                palette::BACKGROUND_COLOR_2.green,
                palette::BACKGROUND_COLOR_2.blue,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
            sdf_param_usage: ShaderParamUsage::NO_PARAMS,
            fill_param_usage: ShaderParamUsage::from_params(&[0,1,2]),
        }
        .with_bundle(Transform {
            scale: Vec3::ONE * scale,
            translation: Vec3::Z * - 10.0,
            ..Default::default()
        });


        commands.add_child(0, node, &());
    }
}