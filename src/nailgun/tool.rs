use bevy::{ecs::query::WorldQuery, prelude::*, render::camera::CameraProjection, sprite::Anchor};
use bevy_rapier2d::prelude::*;

use crate::{
    collision_groups::*,
    map::chunk::Chunkloader,
    packages::presets::Package,
    player::car::Chassis,
    utils::{quat::rot_z, secondary_handle::SecondaryHandle},
};

#[derive(Debug)]
pub struct SelectedItem {
    entity: Entity,
    linear_offset: Vec2,
    angular_offset: f32,
}

#[derive(Debug, Default, Component)]
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

#[derive(Debug, Component)]
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
    packages: Query<(&Transform, &Handle<Image>, &Sprite, &Package)>,
    anchorable: Query<(), With<Anchorable>>,
    mouse: Res<Input<MouseButton>>,
    ctx: Res<RapierContext>,
    colliders: Query<&Collider>,
) -> Option<Vec2> {
    let mut tool = tool.single_mut();
    let position = tool.1.translation.truncate();

    if let Some(item) = tool.0.item.as_ref() {
        if !packages.contains(item.entity) {
            unset_tool(&mut tool.2, &(tool.4 .0), &mut tool.1, &mut tool.3);
            tool.0.item = None;
        }
    }

    if let Some(item) = tool.0.item.as_mut() {
        let collider = colliders.get(item.entity).unwrap();
        let pos = position + item.linear_offset;
        let rot = item.angular_offset;

        let can_place = check_placeable(&ctx, &anchorable, collider, pos, rot);

        let is_point = packages.get(item.entity).unwrap().3.is_point;
        let is_anchor = match is_point {
            true => check_anchor_point(&ctx, &anchorable, position),
            false => check_anchor_shape(&ctx, &anchorable, collider, pos, rot),
        };

        if is_anchor && can_place {
            if mouse.just_pressed(MouseButton::Left) {
                unset_tool(&mut tool.2, &(tool.4 .0), &mut tool.1, &mut tool.3);
                return Some(position);
            } else if tool.3.color != ALPHA_NEUTRAL {
                tool.3.color = ALPHA_NEUTRAL;
            }
        } else if tool.3.color != ALPHA_RED {
            tool.3.color = ALPHA_RED;
        }
    } else if mouse.just_pressed(MouseButton::Left) {
        let entity = check_package(&ctx, position, &packages);
        if let Some(entity) = entity {
            if let Ok((transform, image, sprite, package)) = packages.get(entity) {
                let angular_offset = rot_z(transform.rotation);
                let linear_offset = match package.is_point {
                    true => Vec2::ZERO,
                    false => transform.translation.truncate() - position,
                };

                set_tool(
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

    if mouse.just_pressed(MouseButton::Right) && tool.0.item.is_some() {
        tool.0.item = None;
        unset_tool(&mut tool.2, &(tool.4 .0), &mut tool.1, &mut tool.3);
    }

    None
}

fn check_package<Q: WorldQuery, F: WorldQuery>(
    ctx: &RapierContext,
    position: Vec2,
    packages: &Query<Q, F>,
) -> Option<Entity> {
    let mut entity = None;
    ctx.intersections_with_point(
        position,
        QueryFilter::new().predicate(&|e| packages.contains(e)),
        |e| {
            entity = Some(e);
            false
        },
    );
    entity
}

fn check_anchor_point(
    ctx: &RapierContext,
    anchorable: &Query<(), With<Anchorable>>,
    position: Vec2,
) -> bool {
    let mut is_anchor = false;
    ctx.intersections_with_point(
        position,
        QueryFilter::new().predicate(&|e| anchorable.contains(e)),
        |_| {
            is_anchor = true;
            false
        },
    );
    is_anchor
}

fn check_anchor_shape(
    ctx: &RapierContext,
    anchorable: &Query<(), With<Anchorable>>,
    collider: &Collider,
    pos: Vec2,
    rot: f32,
) -> bool {
    let mut is_anchor = false;
    ctx.intersections_with_shape(
        pos,
        rot,
        collider,
        QueryFilter::new().predicate(&|e| anchorable.contains(e)),
        |_| {
            is_anchor = true;
            false
        },
    );
    is_anchor
}

fn check_placeable(
    ctx: &Res<RapierContext>,
    anchorable: &Query<(), With<Anchorable>>,
    collider: &Collider,
    pos: Vec2,
    rot: f32,
) -> bool {
    let mut overlap = 0u32;
    ctx.intersections_with_shape(
        pos,
        rot,
        collider,
        QueryFilter::new().predicate(&|e| anchorable.contains(e)),
        |_| {
            overlap += 1;
            overlap < 2
        },
    );
    overlap < 2
}

const ALPHA_RED: Color = Color::rgba(1., 0., 0., 0.3);
const ALPHA_NEUTRAL: Color = Color::rgba(1., 1., 1., 0.3);

#[allow(clippy::too_many_arguments)]
fn set_tool(
    tool_image: &mut Handle<Image>,
    image: &Handle<Image>,
    tool_transform: &mut Transform,
    transform: &Transform,
    tool_sprite: &mut Sprite,
    sprite: &Sprite,
    linear_offset: Vec2,
    angular_offset: f32,
) {
    tool_transform.rotation = transform.rotation;
    *tool_image = image.clone();
    let size = sprite.custom_size.unwrap();
    let anchor = Vec2::from_angle(-angular_offset).rotate(-linear_offset);
    tool_sprite.anchor = Anchor::Custom(anchor / size);
    tool_sprite.color = ALPHA_NEUTRAL;
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

pub fn nail(
    In(position): In<Option<Vec2>>,
    mut commands: Commands,
    chassis: Query<(Entity, &Transform), With<Chassis>>,
    mut tool: Query<&mut Nailgun>,
    mut packages: Query<(&mut Transform, &Package), Without<Chassis>>,
    mut z_sequencer: ResMut<ZSequencer>,
) {
    if let Some(position) = position {
        let tool = &mut tool.single_mut();
        let item = &mut tool.item.as_ref().unwrap();
        let (chassis_entity, chassis_transform) = chassis.single();

        let angular_offset = rot_z(chassis_transform.rotation);
        let linear_offset = chassis_transform.translation.truncate() - position;

        let (mut package_transform, package) = packages.get_mut(item.entity).unwrap();

        let joint = match package.is_point {
            true => revolute_joint(linear_offset, angular_offset),
            false => fixed_joint(
                item.linear_offset,
                linear_offset,
                item.angular_offset,
                angular_offset,
            ),
        };

        package_transform.rotation = Quat::from_rotation_z(item.angular_offset);
        package_transform.translation = (position + item.linear_offset).extend(z_sequencer.next());

        commands
            .entity(item.entity)
            .insert(CollisionGroups::new(PLAYER, SOLID_TERRAIN | LOOSE_ITEMS))
            .insert(MultibodyJoint::new(chassis_entity, joint))
            .insert(Anchorable)
            .remove::<Package>();

        tool.item = None;
    }
}

fn fixed_joint(a1: Vec2, a2: Vec2, b1: f32, b2: f32) -> GenericJoint {
    let mut joint = FixedJoint::new();

    let a1 = a1.rotate(Vec2::from_angle(-b1));
    let a2 = a2.rotate(Vec2::from_angle(-b2));

    joint.set_local_anchor1(-a2);
    joint.set_local_anchor2(-a1);
    joint.set_local_basis1(-b2);
    joint.set_local_basis2(-b1);

    joint.into()
}

// todo mini 24.08.2022 - figure out why revolute joint crashes
fn revolute_joint(a2: Vec2, b2: f32) -> GenericJoint {
    let mut joint = RevoluteJoint::new();

    let a2 = a2.rotate(Vec2::from_angle(-b2));

    joint.set_local_anchor1(-a2);

    joint.into()
}
