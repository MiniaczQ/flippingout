use bevy::{prelude::*, render::camera::CameraProjection};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::{
    CollisionGroups, FixedJoint, ImpulseJoint, QueryFilter, RapierContext,
};

use crate::{map::chunk::Chunkloader, packages::presets::Package, player::car::Chassis};

#[derive(Debug, Inspectable)]
pub struct SelectedItem {
    entity: Entity,
    transform: Transform,
}

#[derive(Debug, Default, Component, Inspectable)]
pub struct Nailgun {
    item: Option<SelectedItem>,
}

pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("tool.png");
    commands
        .spawn_bundle(SpriteBundle {
            texture,
            ..Default::default()
        })
        .insert(Nailgun::default());
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn click(
    mut commands: Commands,
    ctx: Res<RapierContext>,
    windows: Res<Windows>,
    camera: Query<(&Transform, &OrthographicProjection), (With<Chunkloader>, Without<Package>)>,
    chassis: Query<(), With<Chassis>>,
    mut packages: Query<&mut Transform, (With<Package>, Without<Chunkloader>, Without<Nailgun>)>,
    mut tool: Query<(&mut Nailgun, &mut Transform), (Without<Chunkloader>, Without<Package>)>,
    mouse: Res<Input<MouseButton>>,
) {
    let window = windows.primary();
    if let Some(cursor) = window.cursor_position() {
        let normalized_screen_position =
            cursor / Vec2::new(window.width() as f32, window.height() as f32) * 2. - 1.;
        let (camera_transform, camera_projection) = camera.single();
        let screen_to_world =
            camera_transform.compute_matrix() * camera_projection.get_projection_matrix().inverse();
        let position = screen_to_world.project_point3(normalized_screen_position.extend(-1.));
        let mut tool = tool.single_mut();
        tool.1.translation = position;
        if mouse.just_pressed(MouseButton::Left) {
            if let Some((entity, _)) =
                ctx.project_point(position.truncate(), true, QueryFilter::default())
            {
                match &tool.0.item {
                    Some(selected_item) => {
                        if chassis.get(entity).is_ok() {
                            if let Ok(mut package_transform) =
                                packages.get_mut(selected_item.entity)
                            {
                                let mut joint = FixedJoint::new();

                                commands
                                    .entity(selected_item.entity)
                                    .insert(CollisionGroups::new(0, 0))
                                    .insert(ImpulseJoint::new(entity, joint));
                                //*package_transform = selected_item.transform;
                                //commands.entity(entity).add_child(selected_item.entity);
                            }
                        }
                        tool.0.item = None;
                    }
                    None => {
                        if packages.get(entity).is_ok() {
                            tool.0.item = Some(SelectedItem {
                                entity,
                                transform: Transform::default(),
                            });
                        }
                    }
                }
            } else {
                println!("projection missed");
                tool.0.item = None;
            }
        }
    }
}
