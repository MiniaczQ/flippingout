use bevy::prelude::Plugin;

use self::director::{defroster, despawn, freezer, spawn, PackageSpawner, PackageSpawnerConfig};

pub mod director;
pub mod presets;

pub struct PackagePlugin;

impl Plugin for PackagePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<PackageSpawnerConfig>()
            .init_resource::<PackageSpawner>()
            .add_system(spawn)
            .add_system(despawn)
            .add_system(freezer)
            .add_system(defroster);
    }
}
