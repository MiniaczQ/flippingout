mod map;
mod utils;

use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use map::chunk::{ChunkPlugin, Chunkloader};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ChunkPlugin)
        .add_startup_system(init)
        .add_system(movement)
        .run();
}

fn init(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(Chunkloader);
}

fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Chunkloader>>,
) {
    let mut dx = 0.;

    if keyboard_input.pressed(KeyCode::Left) {
        dx -= 20.;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        dx += 20.;
    }

    query.for_each_mut(|mut t| t.translation.x += dx);
}
