use std::collections::VecDeque;

pub use bevy::prelude::*;

use crate::startup;

pub struct MotionBlurPlugin;

impl Plugin for MotionBlurPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, track_motion_blur);
    }
}

#[derive(Debug, Component)]
pub struct MotionBlur {
    wait: usize,
    parent_entity: Entity,
    translation_queue: VecDeque<Vec3>, //TODO support rotation
}

impl MotionBlur {
    pub fn new(wait: usize, parent_entity: Entity) -> Self {
        Self {
            wait,
            parent_entity,
            translation_queue: VecDeque::with_capacity(wait),
        }
    }
}

fn track_motion_blur(
    mut commands: Commands,
    mut query: Query<(&mut MotionBlur, &mut Transform, &mut Visibility, Entity)>,
    parents: Query<&GlobalTransform, Without<MotionBlur>>,
) {
    //info!("Blurring");
    let mut count = 0usize;
    for (mut blur, mut transform, mut visibility, entity) in query.iter_mut() {
        count += 1;
        if blur.wait == 0 {
            visibility.set_if_neq(Visibility::Visible);
            if let Some(old_parent_translation) = blur.translation_queue.pop_front() {
                //info!("Blur move");
                transform.translation = old_parent_translation;
            } else {
                //info!("Blur despawn");
                commands.entity(entity).despawn_recursive();
            }
        } else {
            //info!("Blur wait");
            visibility.set_if_neq(Visibility::Hidden);
            blur.wait -= 1;
        }

        if let Ok(parent_translation) = parents.get(blur.parent_entity).map(|x| x.translation()) {
            blur.translation_queue.push_back(parent_translation);
        }
    }
    if count > 1 {
        startup::ADDITIONAL_TRACKING.fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }
}
