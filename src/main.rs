mod map;
mod nailgun;
mod packages;
mod player;
mod utils;

use bevy::{
    prelude::*,
    render::{
        render_resource::SamplerDescriptor,
        texture::{ImageSampler, ImageSettings},
    },
};
use bevy_editor_pls::prelude::*;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use map::chunk::ChunkPlugin;
use nailgun::ToolPlugin;
use packages::{presets::PRESETS, PackagePlugin};
use player::PlayerPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            fit_canvas_to_parent: true,
            ..default()
        })
        .insert_resource(ImageSettings {
            default_sampler: ImageSampler::nearest_descriptor(),
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ChunkPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(PackagePlugin)
        .add_plugin(ToolPlugin)
        .add_system(toggle_debug_render)
        .add_startup_system(init)
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

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    PRESETS[0].apply(
        &mut commands.spawn_bundle(TransformBundle::from(
            Transform::from_xyz(200., 50., 0.).with_rotation(Quat::from_rotation_z(1.7)),
        )),
        &asset_server,
    );
}
