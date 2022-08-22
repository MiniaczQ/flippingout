use bevy::prelude::Plugin;

use self::tool::{click, init};

pub mod tool;

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(init).add_system(click);
    }
}
