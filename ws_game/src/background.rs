use crate::prelude::*;
use bevy_smud::{param_usage::{ShaderParamUsage, ShaderParameter}};
use maveric::{prelude::*, with_bundle::CanWithBundle};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.register_maveric::<Background>();
    }
}

#[derive(Debug, Default, MavericRoot)]
pub struct Background;

impl MavericRootChildren for Background {
    type Context = (MyWindowSize, VideoResource);

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.1.is_streaming {
            //Don't show background when video is streaming
            return;
        }


    const FILL_PARAMETERS: &'static [ShaderParameter] = &[
        ShaderParameter::f32(0),
        ShaderParameter::f32(1),
        ShaderParameter::f32(2),
    ];

        let size = context.0.as_ref();

        let scale = size.scaled_width.max(size.scaled_height);
        let node = SmudShapeNode {
            color: palette::BACKGROUND_COLOR_1.convert_color(),
            sfd: ANYWHERE_SHADER_PATH,
            fill: VORONOI_SHADER_PATH,
            frame_size: 1.0,
            f_params: [
                palette::BACKGROUND_COLOR_2.red.into(),
                palette::BACKGROUND_COLOR_2.green.into(),
                palette::BACKGROUND_COLOR_2.blue.into(),
                0.0,
                0.0,
                0.0,
            ],
            u_params: Default::default(),
            sdf_param_usage: ShaderParamUsage::NO_PARAMS,
            fill_param_usage: ShaderParamUsage(FILL_PARAMETERS),
        }
        .with_bundle(Transform {
            scale: Vec3::ONE * scale,
            translation: Vec3::Z * -10.0,
            ..Default::default()
        });

        commands.add_child(0, node, &());
    }
}
