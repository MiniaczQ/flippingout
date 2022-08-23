use bevy::prelude::*;

use self::tool::{follow_cursor, init, nail, update_state, ZSequencer};

pub mod tool;

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ZSequencer>()
            .add_startup_system(init)
            .add_system(follow_cursor)
            .add_system(update_state.chain(nail).after(follow_cursor));
    }
}
