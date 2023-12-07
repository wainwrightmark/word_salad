use crate::prelude::*;
use bevy_smud::param_usage::{ShaderParamUsage, ShaderParameter};
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
        if context.1.is_selfie_mode {
            //Don't show background when video is streaming
            return;
        }

        const FILL_PARAMETERS: &[ShaderParameter] = &[
            ShaderParameter::f32(0),
            ShaderParameter::f32(1),
            ShaderParameter::f32(2),
            ShaderParameter::f32(3),
        ];

        let size = context.0.as_ref();

        let scale = size.scaled_width.max(size.scaled_height);
        let node = SmudShapeNode {
            color: palette::BACKGROUND_COLOR_1.convert_color(),
            sfd: CIRCLE_SHADER_PATH,
            fill: GRADIENT_SHADER_PATH,
            frame_size: 1.0,
            f_params: [
                0.75,
                palette::BACKGROUND_COLOR_2.red,
                palette::BACKGROUND_COLOR_2.green,
                palette::BACKGROUND_COLOR_2.blue,
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
