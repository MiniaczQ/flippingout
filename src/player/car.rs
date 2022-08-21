use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{
    prelude::{
        AdditionalMassProperties, CoefficientCombineRule, Collider, ExternalForce, Friction,
        GenericJoint, GravityScale, ImpulseJoint, RigidBody, VHACDParameters,
    },
    rapier::prelude::{JointAxesMask, JointAxis},
};

#[derive(Debug, Component, Inspectable)]
pub struct Chassis;

#[derive(Debug, Component, Inspectable)]
pub struct Wheel;

pub fn spawn_player_car(mut commands: Commands) {
    let chassis_scale = 8.;
    let chassis_vert = vec![
        Vec2::new(4., 3.) * chassis_scale,
        Vec2::new(5., 0.) * chassis_scale,
        Vec2::new(11., -1.) * chassis_scale,
        Vec2::new(11., -4.) * chassis_scale,
        Vec2::new(9., -4.) * chassis_scale,
        Vec2::new(7., -2.) * chassis_scale,
        Vec2::new(5., -2.) * chassis_scale,
        Vec2::new(3., -4.) * chassis_scale,
        Vec2::new(-3., -4.) * chassis_scale,
        Vec2::new(-5., -2.) * chassis_scale,
        Vec2::new(-7., -2.) * chassis_scale,
        Vec2::new(-9., -4.) * chassis_scale,
        Vec2::new(-11., -4.) * chassis_scale,
        Vec2::new(-11., 0.) * chassis_scale,
        Vec2::new(-6., 1.) * chassis_scale,
        Vec2::new(-5., 3.) * chassis_scale,
        Vec2::new(4., 3.) * chassis_scale,
    ];
    let chassis = Collider::convex_decomposition_with_params(
        &chassis_vert,
        &(0..chassis_vert.len())
            .map(|i| [i as u32, ((i + 1) % chassis_vert.len()) as u32])
            .collect::<Vec<_>>(),
        &VHACDParameters {
            concavity: 0.01,
            ..Default::default()
        },
    );

    let chassis = commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0., 500., 0.)))
        .insert(RigidBody::Dynamic)
        .insert(chassis)
        .insert(Chassis)
        .insert(ExternalForce::default())
        .insert(AdditionalMassProperties::Mass(40.))
        .insert(GravityScale(5.))
        .id();

    let left_wheel = Collider::ball(15.);

    let mut left_joint = GenericJoint::new(JointAxesMask::Y);
    left_joint.set_local_anchor1(Vec2::new(45., -20.));
    left_joint.set_local_axis1(Vec2::new(0., 1.));
    left_joint.set_limits(JointAxis::X, [-20., 0.]);
    left_joint.set_motor_position(JointAxis::X, -20., 400., 40.);
    left_joint.set_contacts_enabled(false);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(45., 480., 0.)))
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
        });

    let right_wheel = Collider::ball(15.);

    let mut right_joint = GenericJoint::new(JointAxesMask::Y);
    right_joint.set_local_anchor1(Vec2::new(-45., -20.));
    right_joint.set_local_axis1(Vec2::new(0., 1.));
    right_joint.set_limits(JointAxis::X, [-20., 0.]);
    right_joint.set_motor_position(JointAxis::X, -20., 400., 40.);
    right_joint.set_contacts_enabled(false);

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(-45., 480., 0.)))
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
        });
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

    wheels.for_each_mut(|mut f| f.torque = delta);
    chassis.for_each_mut(|mut f| f.torque = delta * 3.);
}
