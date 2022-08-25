use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::{
    prelude::{
        AdditionalMassProperties, CoefficientCombineRule, Collider, CollisionGroups, ExternalForce,
        Friction, GenericJoint, GravityScale, MultibodyJoint, RigidBody,
    },
    rapier::prelude::{JointAxesMask, JointAxis},
};

use crate::{collision_groups::*, nailgun::tool::Anchorable};

#[derive(Debug, Component)]
pub struct Chassis;

#[derive(Debug, Component)]
pub struct Wheel;

pub fn spawn_player_car(mut commands: Commands, asset_server: Res<AssetServer>) {
    let y = 500.;
    let chassis_texture = asset_server.load::<Image, _>("car.png");

    let cs = 8.;
    let chassis_bottom = Collider::convex_hull(&[
        Vec2::new(5.3, 0.) * cs,
        Vec2::new(10.7, -1.) * cs,
        Vec2::new(10.7, -4.) * cs,
        Vec2::new(-11., -4.) * cs,
        Vec2::new(-11., 0.) * cs,
    ])
    .unwrap();

    let chassis_top = Collider::convex_hull(&[
        Vec2::new(3.8, 3.) * cs,
        Vec2::new(5.3, 0.) * cs,
        Vec2::new(-2., 0.) * cs,
        Vec2::new(-2., 3.) * cs,
    ])
    .unwrap();

    let chassis = Collider::compound(vec![
        (Vec2::ZERO, 0., chassis_bottom),
        (Vec2::ZERO, 0., chassis_top),
    ]);

    let chassis = commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0., y, 0.)))
        .insert(RigidBody::Dynamic)
        .insert(chassis)
        .insert(CollisionGroups::new(PLAYER, SOLID_TERRAIN | LOOSE_ITEMS))
        .insert(Chassis)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(40.))
        .insert(Anchorable)
        .insert(Sprite {
            custom_size: Some(Vec2::new(180., 60.)),
            anchor: Anchor::Custom(Vec2::new(0., 0.08)),
            ..Default::default()
        })
        .insert(chassis_texture)
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
        .id();

    let left_wheel = Collider::ball(15.);

    let left_wheel_image = asset_server.load::<Image, _>("wheel1.png");

    let mut left_joint = GenericJoint::new(JointAxesMask::Y);
    left_joint.set_local_anchor1(Vec2::new(45., -20.));
    left_joint.set_local_axis1(Vec2::new(0., 1.));
    left_joint.set_limits(JointAxis::X, [-20., 0.]);
    left_joint.set_motor_position(JointAxis::X, -20., 400., 40.);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(45., y - 20., 0.)))
        .insert(RigidBody::Dynamic)
        .insert(left_wheel)
        .insert(CollisionGroups::new(PLAYER, SOLID_TERRAIN | LOOSE_ITEMS))
        .insert(Wheel)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(10.))
        .insert(MultibodyJoint::new(chassis, left_joint))
        .insert(Friction {
            coefficient: 1.,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Sprite {
            custom_size: Some(Vec2::new(34., 34.)),
            ..Default::default()
        })
        .insert(left_wheel_image)
        .insert(Visibility::default())
        .insert(ComputedVisibility::default());

    let right_wheel = Collider::ball(15.);

    let right_wheel_image = asset_server.load::<Image, _>("wheel2.png");

    let mut right_joint = GenericJoint::new(JointAxesMask::Y);
    right_joint.set_local_anchor1(Vec2::new(-45., -20.));
    right_joint.set_local_axis1(Vec2::new(0., 1.));
    right_joint.set_limits(JointAxis::X, [-20., 0.]);
    right_joint.set_motor_position(JointAxis::X, -20., 400., 40.);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(
            -45.,
            y - 20.,
            0.,
        )))
        .insert(RigidBody::Dynamic)
        .insert(right_wheel)
        .insert(CollisionGroups::new(PLAYER, SOLID_TERRAIN | LOOSE_ITEMS))
        .insert(Wheel)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(10.))
        .insert(MultibodyJoint::new(chassis, right_joint))
        .insert(Friction {
            coefficient: 1.,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Sprite {
            custom_size: Some(Vec2::new(34., 34.)),
            ..Default::default()
        })
        .insert(right_wheel_image)
        .insert(Visibility::default())
        .insert(ComputedVisibility::default());
}

pub fn movement(
    mut wheels: Query<&mut ExternalForce, (With<Wheel>, Without<Chassis>)>,
    mut anchorables: Query<&mut ExternalForce, (With<Anchorable>, Without<Wheel>)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut delta = 0.;

    if keyboard_input.pressed(KeyCode::A) {
        delta += 5.0;
    }

    if keyboard_input.pressed(KeyCode::D) {
        delta -= 5.0;
    }

    wheels.for_each_mut(|mut f| f.torque = delta * 3.);
    anchorables.for_each_mut(|mut f| f.torque = delta * 3.);
}
