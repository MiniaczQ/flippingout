use bevy::{prelude::*, render::camera::CameraProjection};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::{
    CollisionGroups, FixedJoint, ImpulseJoint, QueryFilter, RapierContext,
};

use crate::{map::chunk::Chunkloader, packages::presets::Package, player::car::Chassis};

#[derive(Debug, Inspectable)]
pub struct SelectedItem {
    entity: Entity,
    linear_offset: Vec2,
    angular_offset: f32,
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
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Nailgun::default());
}

pub fn follow_cursor(
    windows: Res<Windows>,
    mut tool: Query<&mut Transform, (With<Nailgun>, Without<Chunkloader>)>,
    camera: Query<(&Transform, &OrthographicProjection), With<Chunkloader>>,
) {
    let window = windows.primary();
    if let Some(cursor) = window.cursor_position() {
        let normalized_screen_position =
            cursor / Vec2::new(window.width() as f32, window.height() as f32) * 2. - 1.;
        let (camera_transform, camera_projection) = camera.single();
        let screen_to_world =
            camera_transform.compute_matrix() * camera_projection.get_projection_matrix().inverse();
        let mut position = screen_to_world.project_point3(normalized_screen_position.extend(-1.));
        position.z = 0.;
        let mut tool = tool.single_mut();
        tool.translation = position;
    }
}

#[derive(Debug, Component, Inspectable)]
pub struct Anchorable;

pub fn update_state(
    mut tool: Query<(&mut Nailgun, &Transform), Without<Package>>,
    packages: Query<&Transform, With<Package>>,
    anchorable: Query<(), With<Anchorable>>,
    mouse: Res<Input<MouseButton>>,
    ctx: Res<RapierContext>,
) -> Option<Vec2> {
    let (mut nailgun, transform) = tool.single_mut();
    let position = transform.translation.truncate();

    if mouse.just_pressed(MouseButton::Left) {
        let result = ctx.project_point(position, true, QueryFilter::default());
        if let Some((entity, projection)) = result {
            if projection.is_inside {
                match &mut nailgun.item {
                    Some(_) => {
                        if anchorable.contains(entity) {
                            println!("nailed");
                            return Some(position);
                        }
                    }
                    None => {
                        if let Ok(transform) = packages.get(entity) {
                            let linear_offset = transform.translation.truncate() - position;
                            let (_, angular_offset) = transform.rotation.to_axis_angle();
                            println!("selected");
                            nailgun.item = Some(SelectedItem {
                                entity,
                                linear_offset,
                                angular_offset,
                            })
                        }
                    }
                }
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) && nailgun.item.is_some() {
        println!("unselected");
        nailgun.item = None;
    }

    None
}

pub fn try_weld(
    In(position): In<Option<Vec2>>,
    mut commands: Commands,
    chassis: Query<(Entity, &Transform), With<Chassis>>,
    mut tool: Query<&mut Nailgun>,
) {
    if let Some(position) = position {
        let (chassis_entity, chassis_transform) = chassis.single();
        let tool = &mut tool.single_mut();
        let item = &mut tool.item.as_ref().unwrap();

        let chassis_linear_offset = chassis_transform.translation.truncate() - position;
        let (_, chassis_angular_offset) = chassis_transform.rotation.to_axis_angle();

        let mut joint = FixedJoint::new();

        //joint.set_local_anchor2(
        //    item.linear_offset
        //        .rotate(Vec2::from_angle(-chassis_angular_offset)),
        //);
        //joint.set_local_anchor1(
        //    -chassis_linear_offset.rotate(Vec2::from_angle(-item.angular_offset)),
        //);
        joint.set_local_basis1(0.);
        joint.set_local_basis2(0.);
        //println!("{:?}", item.linear_offset);
        //println!("{:?}", chassis_linear_offset);
        //println!("{:?}", item.angular_offset);
        //println!("{:?}", chassis_angular_offset);

        commands
            .entity(item.entity)
            .insert(CollisionGroups::new(0, 0))
            .insert(ImpulseJoint::new(chassis_entity, joint))
            .insert(Anchorable)
            .remove::<Package>();

        tool.item = None;
    }
}
