use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::{distributions::WeightedIndex, prelude::Distribution, Rng};

use crate::collision_groups::*;

pub struct Preset {
    chance: u32,
    factory: for<'w, 's, 'a, 'b, 'c> fn(
        &'b mut EntityCommands<'w, 's, 'a>,
        &'c AssetServer,
    ) -> &'b mut EntityCommands<'w, 's, 'a>,
}

impl Preset {
    pub fn get_random(rng: &mut impl Rng) -> &Preset {
        let dist = WeightedIndex::new(PRESETS.map(|p| p.chance)).unwrap();
        &PRESETS[dist.sample(rng)]
    }

    pub fn apply<'w, 's, 'a, 'b, 'c>(
        &self,
        commands: &'b mut EntityCommands<'w, 's, 'a>,
        assets_server: &'c AssetServer,
    ) -> &'b mut EntityCommands<'w, 's, 'a> {
        base_factory(commands);
        (self.factory)(commands, assets_server)
    }
}

/// zero rotation / dont
/// zero offset / dont
/// point attachment / shape attachment
/// special overlap filters?
///
///
///
///
///
///
///
///
///

pub const PRESETS: [Preset; 4] = [
    Preset {
        chance: 0,
        factory: wooden_crate_factory,
    },
    Preset {
        chance: 0,
        factory: metal_ball_factory,
    },
    Preset {
        chance: 1,
        factory: beach_ball_factory,
    },
    Preset {
        chance: 1,
        factory: ice_cube_factory,
    },
];

#[derive(Debug, Component)]
pub struct Package {
    pub name: &'static str,
    pub price: u32,
    pub is_point: bool,
}

fn base_factory<'w, 's, 'a, 'b>(
    commands: &'b mut EntityCommands<'w, 's, 'a>,
) -> &'b mut EntityCommands<'w, 's, 'a> {
    commands
        .insert(RigidBody::Dynamic)
        .insert(CollisionGroups::new(
            LOOSE_ITEMS,
            SOLID_TERRAIN | LOOSE_ITEMS | PLAYER,
        ))
}

fn wooden_crate_factory<'w, 's, 'a, 'b, 'c>(
    commands: &'b mut EntityCommands<'w, 's, 'a>,
    asset_server: &'c AssetServer,
) -> &'b mut EntityCommands<'w, 's, 'a> {
    let collider = Collider::cuboid(30., 30.);
    commands
        .insert(collider)
        .insert(AdditionalMassProperties::Mass(6.))
        .insert(Package {
            name: "Wooden Crate",
            price: 1,
            is_point: false,
        })
        .insert(Sprite {
            custom_size: Some(Vec2::new(64., 64.)),
            ..Default::default()
        })
        .insert(asset_server.load::<Image, _>("box.png"))
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
}

fn metal_ball_factory<'w, 's, 'a, 'b, 'c>(
    commands: &'b mut EntityCommands<'w, 's, 'a>,
    asset_server: &'c AssetServer,
) -> &'b mut EntityCommands<'w, 's, 'a> {
    let collider = Collider::ball(20.);
    commands
        .insert(collider)
        .insert(AdditionalMassProperties::Mass(10.))
        .insert(Package {
            name: "Metal Ball",
            price: 3,
            is_point: false,
        })
        .insert(Sprite {
            custom_size: Some(Vec2::new(44., 44.)),
            ..Default::default()
        })
        .insert(asset_server.load::<Image, _>("ball.png"))
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
}

fn beach_ball_factory<'w, 's, 'a, 'b, 'c>(
    commands: &'b mut EntityCommands<'w, 's, 'a>,
    asset_server: &'c AssetServer,
) -> &'b mut EntityCommands<'w, 's, 'a> {
    let collider = Collider::ball(40.);
    commands
        .insert(collider)
        .insert(AdditionalMassProperties::Mass(1.))
        .insert(Restitution {
            coefficient: 0.95,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Package {
            name: "Beach Ball",
            price: 3,
            is_point: false,
        })
        .insert(Sprite {
            custom_size: Some(Vec2::new(88., 88.)),
            ..Default::default()
        })
        .insert(asset_server.load::<Image, _>("ball.png"))
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
}

fn ice_cube_factory<'w, 's, 'a, 'b, 'c>(
    commands: &'b mut EntityCommands<'w, 's, 'a>,
    asset_server: &'c AssetServer,
) -> &'b mut EntityCommands<'w, 's, 'a> {
    let collider = Collider::round_cuboid(15., 15., 0.01);
    commands
        .insert(collider)
        .insert(Friction {
            coefficient: 0.01,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(AdditionalMassProperties::Mass(6.))
        .insert(Package {
            name: "Ice Cube",
            price: 1,
            is_point: false,
        })
        .insert(Sprite {
            custom_size: Some(Vec2::new(32., 32.)),
            ..Default::default()
        })
        .insert(asset_server.load::<Image, _>("box.png"))
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
}
