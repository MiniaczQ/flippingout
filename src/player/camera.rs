use bevy::prelude::*;

use crate::map::chunk::Chunkloader;

use super::car::Chassis;

pub fn init_cam(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 10.),
            ..Default::default()
        })
        .insert(Chunkloader);
}

pub fn follow_cam(
    mut cam: Query<&mut Transform, (With<Chunkloader>, Without<Chassis>)>,
    chassis: Query<&Transform, (With<Chassis>, Without<Chunkloader>)>,
) {
    let mut cam = cam.single_mut();
    let chassis = chassis.single();

    let delta = chassis.translation.truncate() - cam.translation.truncate();
    cam.translation += (delta * 0.1).extend(0.);
}
