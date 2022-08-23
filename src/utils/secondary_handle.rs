use bevy::{asset::Asset, prelude::*};

#[derive(Debug, Component, Clone)]
pub struct SecondaryHandle<T: Asset>(pub Handle<T>);
