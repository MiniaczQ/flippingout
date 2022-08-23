use bevy::{prelude::*, render::camera::CameraProjection, sprite::Anchor};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;

use crate::{
    map::chunk::Chunkloader, packages::presets::Package, player::car::Chassis,
    utils::secondary_handle::SecondaryHandle,
};

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
            texture: texture.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Nailgun::default())
        .insert(SecondaryHandle(texture));
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
        tool.single_mut().translation = screen_to_world
            .project_point3(normalized_screen_position.extend(-1.))
            .truncate()
            .extend(10.);
    }
}

#[derive(Debug, Component, Inspectable)]
pub struct Anchorable;

#[derive(Debug, Default)]
pub struct ZSequencer(f32);

impl ZSequencer {
    pub fn next(&mut self) -> f32 {
        self.0 += f32::EPSILON;
        self.0
    }
}

#[allow(clippy::type_complexity)]
pub fn update_state(
    mut tool: Query<
        (
            &mut Nailgun,
            &mut Transform,
            &mut Handle<Image>,
            &mut Sprite,
            &SecondaryHandle<Image>,
        ),
        Without<Package>,
    >,
    packages: Query<(&Transform, &Handle<Image>, &Sprite), With<Package>>,
    anchorable: Query<(), With<Anchorable>>,
    mouse: Res<Input<MouseButton>>,
    ctx: Res<RapierContext>,
    images: Res<Assets<Image>>,
) -> Option<Vec2> {
    let mut tool = tool.single_mut();
    let position = tool.1.translation.truncate();

    if mouse.just_pressed(MouseButton::Left) {
        let result = ctx.project_point(position, true, QueryFilter::default());
        if let Some((entity, projection)) = result {
            if projection.is_inside {
                match &mut tool.0.item {
                    Some(_) => {
                        if anchorable.contains(entity) {
                            unset_tool(&mut tool.2, &(tool.4 .0), &mut tool.1, &mut tool.3);
                            return Some(position);
                        }
                    }
                    None => {
                        if let Ok((transform, image, sprite)) = packages.get(entity) {
                            let (_, angular_offset) = transform.rotation.to_axis_angle();
                            let linear_offset = transform.translation.truncate() - position;

                            set_tool(
                                &images,
                                &mut tool.2,
                                image,
                                &mut tool.1,
                                transform,
                                &mut tool.3,
                                sprite,
                                linear_offset,
                                angular_offset,
                            );

                            tool.0.item = Some(SelectedItem {
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

    if mouse.just_pressed(MouseButton::Right) && tool.0.item.is_some() {
        tool.0.item = None;
        unset_tool(&mut tool.2, &(tool.4 .0), &mut tool.1, &mut tool.3);
    }

    None
}

#[allow(clippy::too_many_arguments)]
fn set_tool(
    images: &Assets<Image>,
    tool_image: &mut Handle<Image>,
    image: &Handle<Image>,
    tool_transform: &mut Transform,
    transform: &Transform,
    tool_sprite: &mut Sprite,
    sprite: &Sprite,
    linear_offset: Vec2,
    angular_offset: f32,
) {
    let image_size = images.get(image).unwrap().size();
    tool_transform.rotation = transform.rotation;
    *tool_image = image.clone();
    tool_sprite.anchor =
        Anchor::Custom(-linear_offset.rotate(Vec2::from_angle(-angular_offset)) * 2. / image_size);
    tool_sprite.color = Color::rgba(1., 1., 1., 0.3);
    tool_sprite.custom_size = sprite.custom_size;
}

fn unset_tool(
    tool_image: &mut Handle<Image>,
    backup_image: &Handle<Image>,
    tool_transform: &mut Transform,
    tool_sprite: &mut Sprite,
) {
    tool_transform.rotation = Quat::IDENTITY;
    *tool_image = backup_image.clone();
    tool_sprite.anchor = Anchor::BottomLeft;
    tool_sprite.color = Color::WHITE;
    tool_sprite.custom_size = Some(Vec2::new(50., 50.));
}

pub fn try_weld(
    In(position): In<Option<Vec2>>,
    mut commands: Commands,
    chassis: Query<(Entity, &Transform), With<Chassis>>,
    mut tool: Query<&mut Nailgun>,
    mut items: Query<&mut Transform, (With<Package>, Without<Chassis>)>,
    mut z_sequencer: ResMut<ZSequencer>,
) {
    if let Some(position) = position {
        let (entity, transform) = chassis.single();
        let tool = &mut tool.single_mut();
        let item = &mut tool.item.as_ref().unwrap();

        let (_, angular_offset) = transform.rotation.to_axis_angle();
        let linear_offset = transform.translation.truncate() - position;

        let joint = construct_joint(
            item.linear_offset,
            linear_offset,
            item.angular_offset,
            angular_offset,
        );

        let mut item_transform = items.get_mut(item.entity).unwrap();

        item_transform.rotation = Quat::from_rotation_z(item.angular_offset);
        item_transform.translation = (position + item.linear_offset).extend(z_sequencer.next());

        commands
            .entity(item.entity)
            .insert(CollisionGroups::new(0, 0))
            .insert(MultibodyJoint::new(entity, joint))
            .insert(Anchorable)
            .remove::<Package>();

        tool.item = None;
    }
}

fn construct_joint(a1: Vec2, a2: Vec2, b1: f32, b2: f32) -> FixedJoint {
    let mut joint = FixedJoint::new();

    let a1 = a1.rotate(Vec2::from_angle(-b1));
    let a2 = a2.rotate(Vec2::from_angle(-b2));

    joint.set_local_anchor1(-a2);
    joint.set_local_anchor2(-a1);
    joint.set_local_basis1(-b2);
    joint.set_local_basis2(-b1);

    joint
}
