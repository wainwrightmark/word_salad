use std::time::Duration;

use maveric::{helpers::ChildCommands, node::MavericNode, with_bundle::CanWithBundle};

use crate::prelude::*;

#[derive(Debug, PartialEq, MavericRoot)]
pub struct SelfiePopup {}

impl MavericRootChildren for SelfiePopup {
    type Context = (MyWindowSize, SelfieModeHistory);

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child(
            0,
            SelfiePopupNode {
                has_entered_selfie_mode: context.1.has_entered_selfie_mode,
            },
            &context.0,
        )
    }
}

#[derive(Debug, PartialEq)]
struct SelfiePopupNode {
    pub has_entered_selfie_mode: bool,
}

impl MavericNode for SelfiePopupNode {
    type Context = MyWindowSize;

    fn set_components(mut commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_static_bundle(SpatialBundle::default());
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.unordered(|args, commands| {
            if args.node.has_entered_selfie_mode
                && args.previous.is_some_and(|x| !x.has_entered_selfie_mode)
            {
                commands.add_child(
                    "top",
                    TutorialPopupNode {
                        text: "Welcome to Selfie Mode!\nTry using a screen recorder app",
                        entity: TutorialLayoutEntity::Top,
                    }
                    .with_bundle(ScheduledForDeletion {
                        remaining: Duration::from_secs(5),
                    })
                    .with_transition_in::<TransformScaleLens>(
                        Vec3::ZERO,
                        Vec3::ONE,
                        Duration::from_secs_f32(0.5),
                    ),
                    args.context,
                );
            }
        })
    }
}
