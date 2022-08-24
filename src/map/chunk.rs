use bevy::{
    prelude::*, render::render_resource::PrimitiveTopology, sprite::MaterialMesh2dBundle,
    utils::HashSet,
};
use bevy_rapier2d::{prelude::*, rapier::prelude::Vector};
use itertools::{izip, repeat_n, Itertools};
use rand::{distributions::Uniform, prelude::Distribution, rngs::SmallRng, *};

use crate::{collision_groups::*, utils::iter::IteratorExt};

#[derive(Debug, Component)]
pub struct Chunk(i32);

#[derive(Debug, Component)]
pub struct Chunkloader;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ChunkGen {
    // sin noise
    frequencies: [f32; 10],
    phases: [f32; 10],
    amplitudes: [f32; 10],
    // scaling
    amplitude_scaling: Box<fn(f32) -> f32>,
    amplitude_scaling_derivative: Box<fn(f32) -> f32>,
}

impl ChunkGen {
    pub fn reset(&mut self, config: &ChunkGenConfig) {
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

    pub fn probe(&self, x: f32) -> f32 {
        self.frequencies
            .iter()
            .zip(self.phases)
            .map(|(f, p)| (x * f + p).sin())
            .zip(self.amplitudes)
            .map(|(s, a)| s * a)
            .sum::<f32>()
        //* (self.amplitude_scaling)(x)
    }

    fn probe_derivative(&self, x: f32) -> f32 {
        let (y, dy) = izip!(self.frequencies, self.phases, self.amplitudes)
            .map(|(f, p, a)| {
                let (s, c) = (x * f + p).sin_cos();
                (s * a, c * f * a)
            })
            .fold((0., 0.), |(ay, ady), (y, dy)| (ay + y, ady + dy));
        //let y2 = (self.amplitude_scaling)(x);
        //let dy2 = (self.amplitude_scaling_derivative)(x);
        //y * dy2 + dy * y2 + dy * dy2
        dy
    }
}

impl Default for ChunkGen {
    fn default() -> Self {
        Self {
            frequencies: Default::default(),
            phases: Default::default(),
            amplitudes: Default::default(),
            amplitude_scaling: Box::new(|x| {
                let x = (x - 1000.) / 1000.;
                if x > 1. {
                    x.sqrt()
                } else if x > 0. {
                    x * x
                } else {
                    0.
                }
            }),
            amplitude_scaling_derivative: Box::new(|x| {
                let x = (x - 1000.) / 1000.;
                if x > 1. {
                    1. / (2. * x.sqrt())
                } else if x > 0. {
                    x
                } else {
                    0.
                }
            }),
        }
    }
}

#[derive(Debug)]
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

fn generate_chunks(
    mut commands: Commands,
    config: Res<ChunkConfig>,
    gen: Res<ChunkGen>,
    chunks: Query<&Chunk, (With<Chunk>, Without<Chunkloader>)>,
    chunkloaders: Query<&Transform, (With<Chunkloader>, Without<Chunk>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
        generate_chunk(
            &mut commands,
            &mut materials,
            &mut meshes,
            &config,
            &gen,
            x,
            i,
        );
    });
}

const GRASS_COLOR: Color = Color::rgba(0.3, 1., 0.3, 1.);
const EARTH_COLOR: Color = Color::rgba(0.5, 0.3, 0.3, 1.);

fn generate_chunk(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut Assets<Mesh>,
    config: &ChunkConfig,
    gen: &ChunkGen,
    x: f32,
    i: i32,
) {
    let (collider, grass_mesh, earth_mesh) = generate_meshes(meshes, config, gen, x);

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: grass_mesh.into(),
            material: materials.add(ColorMaterial::from(GRASS_COLOR)),
            transform: Transform::from_xyz(x, 0., -1.),
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(collider)
        .insert(CollisionGroups::new(SOLID_TERRAIN, LOOSE_ITEMS | PLAYER))
        .insert(Chunk(i))
        .with_children(|b| {
            b.spawn_bundle(MaterialMesh2dBundle {
                mesh: earth_mesh.into(),
                material: materials.add(ColorMaterial::from(EARTH_COLOR)),
                transform: Transform::from_xyz(0., 0., -1.),
                ..Default::default()
            });
        });
}

fn generate_meshes(
    meshes: &mut Assets<Mesh>,
    config: &ChunkConfig,
    gen: &ChunkGen,
    x: f32,
) -> (Collider, Handle<Mesh>, Handle<Mesh>) {
    let offset = 10.;
    let probes = config.probes as usize;
    let half_size = config.x_size / 2.;
    let dx = config.x_size / (config.probes - 1) as f32;
    let (y, pos, norm, pos2): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = (0..config.probes)
        .map(|i| {
            let lx = i as f32 * dx;
            let gx = x + lx;
            let y = gen.probe(gx);
            let pos = [lx - half_size, y, 0.];
            let norm =
                Vec2::from_angle(gen.probe_derivative(gx).atan2(dx) + std::f32::consts::FRAC_PI_2);
            let norm = [norm.x, norm.y, 0.];
            let pos2 = Vec2::new(pos[0] - norm[0] * offset, pos[1] - norm[1] * offset);
            let pos2 = [pos2.x, pos2.y, 0.];
            (y, pos, norm, pos2)
        })
        .multiunzip();

    let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
    let positions = pos
        .iter()
        .interleave(pos2.iter())
        .copied()
        .collect::<Vec<_>>();
    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    let normals = norm
        .iter()
        .copied()
        .interleave(norm.iter().map(|x| [-x[0], -x[1], 0.]))
        .collect::<Vec<_>>();
    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    let uvs = repeat_n([0., 0.], 2 * probes).collect::<Vec<_>>();
    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    let grass_mesh = meshes.add(grass_mesh);

    let mut earth_mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
    let positions = pos2
        .iter()
        .copied()
        .interleave(pos2.iter().map(|p| [p[0], -1000., 0.]))
        .collect::<Vec<_>>();
    earth_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    let normals = norm
        .iter()
        .copied()
        .interleave(repeat_n([0., -1., 0.], probes))
        .collect::<Vec<_>>();
    earth_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    let uvs = repeat_n([0., 0.], 2 * probes).collect::<Vec<_>>();
    earth_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    let earth_mesh = meshes.add(earth_mesh);

    let scale = config.x_size;
    let collider = Collider::heightfield(y, Vector::new(scale, 1.));

    (collider, grass_mesh, earth_mesh)
}

fn init(mut gen: ResMut<ChunkGen>, config: Res<ChunkGenConfig>) {
    gen.reset(&config);
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
    if keyboard_input.just_pressed(KeyCode::P) {
        query.for_each(|e| commands.entity(e).despawn_recursive());
    }
}
