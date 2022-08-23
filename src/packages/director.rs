use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::{RigidBody, Sleeping};
use rand::{rngs::SmallRng, SeedableRng};

use crate::player::car::Chassis;

use super::presets::{Package, Preset};

#[derive(Debug)]
pub struct PackageSpawnerConfig {
    distance_apart: f32,
    spawn_distance: f32,
    despawn_distance: f32,
    freeze_distance: f32,
}

impl Default for PackageSpawnerConfig {
    fn default() -> Self {
        Self {
            distance_apart: 1024.,
            spawn_distance: 2048.,
            despawn_distance: 8192.,
            freeze_distance: 3072.,
        }
    }
}

#[derive(Debug)]
pub struct PackageSpawner {
    rng: SmallRng,
    last_spawned: u32,
}

impl Default for PackageSpawner {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            last_spawned: 0,
        }
    }
}

pub fn spawn(
    mut commands: Commands,
    player: Query<&Transform, With<Chassis>>,
    config: Res<PackageSpawnerConfig>,
    mut spawner: ResMut<PackageSpawner>,
    asset_server: Res<AssetServer>,
) {
    let player_x = player.single().translation.x;
    let last_spawned = spawner.last_spawned as f32 * config.distance_apart;

    if player_x + config.spawn_distance > last_spawned {
        spawner.last_spawned += 1;
        let to_spawn = spawner.last_spawned as f32 * config.distance_apart;
        let preset = Preset::get_random(&mut spawner.rng);
        let mut entity = commands.spawn_bundle(TransformBundle::from(Transform::from_xyz(
            to_spawn, 500., 0.,
        )));
        entity.insert(Sleeping::default());
        preset.apply(&mut entity, &asset_server);
    }
}

#[allow(clippy::type_complexity)]
pub fn despawn(
    mut commands: Commands,
    player: Query<&Transform, (With<Chassis>, Without<Package>)>,
    config: Res<PackageSpawnerConfig>,
    packages: Query<(Entity, &Transform), (With<Package>, Without<Chassis>)>,
) {
    let player_x = player.single().translation.x;

    packages.for_each(|(entity, package_transform)| {
        if player_x - package_transform.translation.x > config.despawn_distance {
            commands.entity(entity).despawn_recursive();
        }
    })
}

#[derive(Debug, Component, Inspectable)]
pub struct Frozen;

#[allow(clippy::type_complexity)]
pub fn freezer(
    mut commands: Commands,
    config: Res<PackageSpawnerConfig>,
    player: Query<&Transform, (With<Chassis>, Without<Package>)>,
    defrosted: Query<(Entity, &Transform), (With<Package>, Without<Chassis>, Without<Frozen>)>,
) {
    let player_x = player.single().translation.x;

    defrosted.for_each(|(entity, package_transform)| {
        if (player_x - package_transform.translation.x).abs() > config.freeze_distance {
            commands
                .entity(entity)
                .insert(Frozen)
                .insert(RigidBody::Fixed);
        }
    })
}

#[allow(clippy::type_complexity)]
pub fn defroster(
    mut commands: Commands,
    config: Res<PackageSpawnerConfig>,
    player: Query<&Transform, (With<Chassis>, Without<Package>)>,
    frozen: Query<(Entity, &Transform), (With<Package>, Without<Chassis>, With<Frozen>)>,
) {
    let player_x = player.single().translation.x;

    frozen.for_each(|(entity, package_transform)| {
        if (player_x - package_transform.translation.x).abs() < config.freeze_distance {
            commands
                .entity(entity)
                .remove::<Frozen>()
                .insert(RigidBody::Dynamic);
        }
    })
}
