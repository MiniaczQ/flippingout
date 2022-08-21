use bevy::prelude::Plugin;

use self::{
    camera::{init_cam, follow_cam},
    car::{movement, spawn_player_car},
};

pub mod camera;
pub mod car;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(init_cam)
            .add_system(follow_cam)
            .add_startup_system(spawn_player_car)
            .add_system(movement);
    }
}
