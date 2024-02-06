use crate::prelude::*;
use maveric::with_bundle::CanWithBundle;
use ws_core::layout::entities::*;
#[derive(Debug, NodeContext)]
pub struct LogoContext {
    pub window_size: MyWindowSize,
    // pub hint_state: HintState,
    pub video_resource: VideoResource,
}

impl<'a, 'w: 'a> From<&'a ViewContextWrapper<'w>> for LogoContextWrapper<'w> {
    fn from(value: &'a ViewContextWrapper<'w>) -> Self {
        Self {
            // hint_state: Res::clone(&value.hint_state),
            window_size: Res::clone(&value.window_size),
            video_resource: Res::clone(&value.video_resource),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WordSaladLogoNode;

impl MavericNode for WordSaladLogoNode {
    type Context = LogoContext;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let size = &context.window_size;

                let logo_rect =
                    size.get_rect(&WordSaladLogo, &context.video_resource.selfie_mode());

                commands.add_child(
                    "Word Salad Icon",
                    SpriteNode {
                        texture_path: r#"images/logo1024.png"#,
                        sprite: Sprite {
                            custom_size: Some(logo_rect.extents.abs()),
                            ..Default::default()
                        },
                    }
                    .with_bundle((Transform::from_translation(
                        logo_rect.centre().extend(crate::z_indices::TOP_BAR_BUTTON),
                    ),)),
                    &(),
                );
            });
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct LogoImageNodeStyle;

impl IntoBundle for LogoImageNodeStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            margin: UiRect::DEFAULT,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        }
    }
}
