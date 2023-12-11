// use bevy::{ecs::system::EntityCommands, prelude::*};
// use maveric::helpers::Query;

// pub struct ScheduledComponentPlugin;

// impl Plugin for ScheduledComponentPlugin {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         app.add_systems(Last, handle_scheduled_components);
//     }
// }

// #[derive(Component)]
// pub struct ScheduledChange {
//     pub timer: Timer,
//     pub boxed_change: Box<dyn FnOnce(&mut EntityCommands) + 'static + Sync + Send>,
// }

// fn handle_scheduled_components(
//     mut commands: Commands,
//     mut query: Query<(Entity, &mut ScheduledChange)>,
//     time: Res<Time>,
// ) {
//     for (entity, mut schedule) in query.iter_mut() {
//         schedule.timer.tick(time.delta());
//         if schedule.timer.finished() {
//             let mut ec = commands.entity(entity);
//             ec.remove::<ScheduledChange>();

//             let mut f: Box<dyn FnOnce(&mut EntityCommands) + 'static + Sync + Send> =
//                 Box::new(|_| {});

//             std::mem::swap(&mut f, &mut schedule.boxed_change);
//             f(&mut ec);
//         }
//     }
// }
