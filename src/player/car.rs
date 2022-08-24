use bevy::{prelude::*, sprite::Anchor};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{
    prelude::{
        AdditionalMassProperties, CoefficientCombineRule, Collider, ExternalForce, Friction,
        GenericJoint, GravityScale, ImpulseJoint, RigidBody,
    },
    rapier::prelude::{JointAxesMask, JointAxis},
};

use crate::nailgun::tool::Anchorable;

#[derive(Debug, Component, Inspectable)]
pub struct Chassis;

#[derive(Debug, Component, Inspectable)]
pub struct Wheel;

pub fn spawn_player_car(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chassis_texture = asset_server.load::<Image, _>("car.png");

    let chassis_scale = 8.;
    let chassis_vert = vec![
        Vec2::new(4., 3.) * chassis_scale,
        Vec2::new(5., 0.) * chassis_scale,
        Vec2::new(11., -1.) * chassis_scale,
        Vec2::new(11., -4.) * chassis_scale,
        Vec2::new(-11., -4.) * chassis_scale,
        Vec2::new(-11., 0.) * chassis_scale,
        Vec2::new(-6., 1.) * chassis_scale,
        Vec2::new(-5., 3.) * chassis_scale,
        Vec2::new(4., 3.) * chassis_scale,
    ];
    let chassis = Collider::convex_decomposition(
        &chassis_vert,
        &(0..chassis_vert.len())
            .map(|i| [i as u32, ((i + 1) % chassis_vert.len()) as u32])
            .collect::<Vec<_>>(),
    );

    let chassis = commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0., 200., 0.)))
        .insert(RigidBody::Dynamic)
        .insert(chassis)
        .insert(Chassis)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(40.))
        .insert(GravityScale(5.))
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
    left_joint.set_contacts_enabled(false);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(45., 80., 0.)))
        .insert(RigidBody::Dynamic)
        .insert(left_wheel)
        .insert(Wheel)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(10.))
        .insert(GravityScale(5.))
        .insert(ImpulseJoint::new(chassis, left_joint))
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
    right_joint.set_contacts_enabled(false);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(-45., 80., 0.)))
        .insert(RigidBody::Dynamic)
        .insert(right_wheel)
        .insert(Wheel)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(10.))
        .insert(GravityScale(5.))
        .insert(ImpulseJoint::new(chassis, right_joint))
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
    mut chassis: Query<&mut ExternalForce, (With<Chassis>, Without<Wheel>)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut delta = 0.;

    if keyboard_input.pressed(KeyCode::Left) {
        delta += 5.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        delta -= 5.0;
    }

    wheels.for_each_mut(|mut f| f.torque = delta * 2.);
    chassis.for_each_mut(|mut f| f.torque = delta * 3.);
}
