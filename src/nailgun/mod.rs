use bevy::prelude::*;

use self::tool::{follow_cursor, init, try_weld, update_state};

pub mod tool;

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(init)
            .add_system(follow_cursor)
            .add_system(update_state.chain(try_weld).after(follow_cursor));
    }
}
