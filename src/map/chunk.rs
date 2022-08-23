use bevy::{prelude::*, utils::HashSet};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{prelude::*, rapier::prelude::Vector};
use rand::{distributions::Uniform, prelude::Distribution, rngs::SmallRng, *};

use crate::utils::iter::IteratorExt;

#[derive(Debug, Component, Inspectable)]
pub struct Chunk(i32);

#[derive(Debug, Component, Inspectable)]
pub struct Chunkloader;

#[derive(Debug, Inspectable)]
pub struct ChunkGenConfig {
    frequency_range: (f32, f32),
    phase_range: (f32, f32),
    amplitude_range: (f32, f32),
}

impl Default for ChunkGenConfig {
    fn default() -> Self {
        Self {
            frequency_range: (0., 0.005),
            phase_range: (0., std::f32::consts::TAU),
            amplitude_range: (30., 36.),
        }
    }
}

#[derive(Debug, Inspectable)]
pub struct ChunkGen {
    // sin noise
    frequencies: [f32; 10],
    phases: [f32; 10],
    amplitudes: [f32; 10],
    // scaling
    #[inspectable(ignore)]
    amplitude_scaling: Box<fn(f32) -> f32>,
    #[inspectable(ignore)]
    height_offset: Box<fn(f32) -> f32>,
}

impl ChunkGen {
    fn reset(&mut self, config: &ChunkGenConfig) {
        let rng = SmallRng::from_entropy();
        self.frequencies = Uniform::new(config.frequency_range.0, config.frequency_range.1)
            .sample_iter(rng)
            .take_array();

        let rng = SmallRng::from_entropy();
        self.phases = Uniform::new(config.phase_range.0, config.phase_range.1)
            .sample_iter(rng)
            .take_array();

        let rng = SmallRng::from_entropy();
        self.amplitudes = Uniform::new(config.amplitude_range.0, config.amplitude_range.1)
            .sample_iter(rng)
            .take_array();
    }

    fn probe(&self, x: f32) -> f32 {
        self.frequencies
            .iter()
            .zip(self.phases)
            .map(|(f, p)| (x * f + p).sin())
            .zip(self.amplitudes)
            .map(|(s, a)| s * a)
            .sum::<f32>()
            * (self.amplitude_scaling)(x)
            + (self.height_offset)(x)
    }
}

impl Default for ChunkGen {
    fn default() -> Self {
        Self {
            frequencies: Default::default(),
            phases: Default::default(),
            amplitudes: Default::default(),
            amplitude_scaling: Box::new(|_| 1.),
            height_offset: Box::new(|_| 0.),
        }
    }
}

#[derive(Debug, Inspectable)]
pub struct ChunkConfig {
    probes: u32,
    x_size: f32,
    gen_distance: f32,
    rem_distance: f32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            probes: 33,
            x_size: 1024.,
            gen_distance: 4096.,
            rem_distance: 8192.,
        }
    }
}

fn calculate_distance(a: &Transform, b: &Transform) -> f32 {
    (a.translation.x - b.translation.x).abs()
}

#[allow(clippy::type_complexity)]
fn remove_chunks(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    chunks: Query<(Entity, &Transform), (With<Chunk>, Without<Chunkloader>)>,
    chunkloaders: Query<&Transform, (With<Chunkloader>, Without<Chunk>)>,
) {
    chunks
        .iter()
        .filter_map(|(chunk_entity, chunk_transform)| {
            let distance = chunkloaders
                .iter()
                .map(|chunkloader_transform| {
                    calculate_distance(chunk_transform, chunkloader_transform)
                })
                .reduce(f32::min)
                .unwrap();
            if distance > config.rem_distance {
                Some(chunk_entity)
            } else {
                None
            }
        })
        .for_each(|entity| commands.entity(entity).despawn_recursive());
}

fn generate_chunk(commands: &mut Commands, config: &ChunkConfig, gen: &ChunkGen, x: f32, i: i32) {
    let dx = config.x_size / (config.probes - 1) as f32;
    let heights = (0..config.probes)
        .map(|i| {
            let x = x + i as f32 * dx;
            gen.probe(x)
        })
        .collect::<Vec<_>>();
    let scale = config.x_size;

    let collider = Collider::heightfield(heights, Vector::new(scale, 1.));

    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(x, 0., 0.)))
        .insert(RigidBody::Fixed)
        .insert(collider)
        .insert(Chunk(i));
}

fn generate_chunks(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    gen: Res<ChunkGen>,
    chunks: Query<&Chunk, (With<Chunk>, Without<Chunkloader>)>,
    chunkloaders: Query<&Transform, (With<Chunkloader>, Without<Chunk>)>,
) {
    let mut missing = HashSet::new();
    chunkloaders.iter().for_each(|chunkloader_transform| {
        let min_i = ((chunkloader_transform.translation.x - config.gen_distance) / config.x_size)
            .floor() as i32;
        let max_i = ((chunkloader_transform.translation.x + config.gen_distance) / config.x_size)
            .ceil() as i32;

        missing.extend(min_i..=max_i);
    });

    chunks.for_each(|chunk| {
        missing.remove(&chunk.0);
    });

    missing.into_iter().for_each(|i| {
        let x = i as f32 * config.x_size;
        generate_chunk(&mut commands, &config, &gen, x, i);
    });
}

fn init(mut gen: ResMut<ChunkGen>, config: Res<ChunkGenConfig>) {
    //gen.reset(&config);
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ChunkGenConfig>()
            .init_resource::<ChunkGen>()
            .init_resource::<ChunkConfig>()
            .add_startup_system(init)
            .add_system(remove_chunks)
            .add_system(generate_chunks)
            .add_system(reset);
    }
}

fn reset(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Chunk>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        query.for_each(|e| commands.entity(e).despawn_recursive());
    }
}
