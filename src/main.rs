mod map;
mod player;
mod utils;

use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use map::chunk::ChunkPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ChunkPlugin)
        .add_plugin(PlayerPlugin)
        .add_system(toggle_debug_render)
        .run();
}

fn toggle_debug_render(
    mut render: ResMut<DebugRenderContext>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::D) {
        render.enabled = !render.enabled;
    }
}
