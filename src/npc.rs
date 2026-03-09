use bevy::math::Vec2;
use bevy::prelude::*;
use std::any::TypeId;

use crate::animation::*;

pub const AREA_SIZE: Vec2 = vec2(1280.0, 720.0);

fn positive_rem(x: f32, div: f32) -> f32 {
    let rem = x % div;
    if rem < 0.0 {
        return rem + div.abs();
    }
    return rem;
}

// Centralise on the center
pub fn stay_in_area(position: Vec2) -> Vec2 {
    Vec2 {
        x: positive_rem(position.x + AREA_SIZE.x / 2.0,  AREA_SIZE.x) - AREA_SIZE.x / 2.0,
        y: positive_rem(position.y + AREA_SIZE.y / 2.0, AREA_SIZE.y) - AREA_SIZE.y / 2.0,
        // x: position.x % (AREA_SIZE.x/2.0),
        // y: position.y % (AREA_SIZE.y/2.0),
    }
}

// mod animation;
use crate::animation::*;

const NPC_SPEED: f32 = 100.0;

#[derive(Component)]
pub struct Despawns {
    timer: Option<Timer>,
}


#[derive(Component)]
pub struct NpcEntity {
    pub speed: f32,
    pub move_direction: Vec2,
    pub moving: bool,
    pub gender: NpcGender,
}

#[derive(Component)]
pub struct FlowerEntity;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NpcGender {
    Male,
    Female,
}

impl NpcEntity {
    pub fn new(gender: NpcGender) -> Self {
        Self {
            speed: NPC_SPEED,
            move_direction: Vec2::new(0.0, 0.0),
            moving: false,
            gender,
        }
    }
}

#[derive(Component)]
pub struct ReceivedFlowers {
    pub has_received: bool,
    pub flower_spawned: bool,
}

impl ReceivedFlowers {
    pub fn new() -> Self {
        Self { 
            has_received: false, 
            flower_spawned: false,
        }
    }
}

pub fn set_npc_movement(
    mut npc_query: Query<(&mut NpcEntity, &ReceivedFlowers)>,
) {
    for (mut npc, received_flowers) in &mut npc_query {
        // Only change direction if NPC hasn't received flowers yet
        if !received_flowers.has_received {
            if rand::random::<f32>() < 0.02 {
                npc.move_direction = Vec2::new(
                    rand::random::<f32>() * 2.0 - 1.0,
                    rand::random::<f32>() * 2.0 - 1.0,
                ).normalize_or_zero();
                npc.moving = npc.move_direction.length() > 0.1;
            }
        } else {
            // If NPC has received flowers, stop moving
            npc.moving = false;
            npc.move_direction = Vec2::new(0.0, 0.0);
        }
    }
}

pub fn move_npcs(time: Res<Time>, mut query: Query<(&mut Transform, &NpcEntity)>) {
    let dt = time.delta_secs();
    for (mut transform, npc) in &mut query {
        if npc.moving {
            let translation_2d = transform.translation.truncate() + npc.move_direction * npc.speed * dt;
            let translation_in_area = stay_in_area(translation_2d);
            transform.translation = Vec3::from_array([translation_in_area.x, translation_in_area.y, 0.0]);
        }
    }
}

pub fn spawn_flower_over_npc(
    mut commands: Commands,
    mut npc_query: Query<(&Transform, &mut ReceivedFlowers, &mut Despawns), With<NpcEntity>>,
    asset_server: Res<AssetServer>,
    // mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Check for NPCs that have received flowers but haven't spawned flower yet
    for (npc_transform, mut received_flowers, mut despawns) in &mut npc_query {
        if received_flowers.has_received && !received_flowers.flower_spawned {
            // Spawn flower sprite above the NPC
            let flower_texture = asset_server.load("textures/rpg/props/generic-rpg-flower01.png");
            
            // Spawn flower sprite with slight offset above NPC
            commands.spawn((
                Sprite {
                    image: flower_texture,
                    custom_size: Some(vec2(2.0*8.0, 2.0*8.0)),
                    ..Default::default()
                },
                Transform::from_scale(Vec3::splat(3.0)).with_translation(Vec3::new(
                    npc_transform.translation.x,
                    npc_transform.translation.y + 45.0, // Offset up so it doesn't collide with NPC
                    1.0
                )),
                // Add a component to identify flower entities for later despawning
                FlowerEntity,
                Despawns { timer: Some(Timer::from_seconds(3.0, TimerMode::Once)) },
                LinearMotion { speed: vec2(0.0, 50.0) }
            ));
            received_flowers.flower_spawned = true;
            match despawns.timer {
                None => despawns.timer = Some(Timer::from_seconds(3.0, TimerMode::Once)),
                _ => (),
            }
        }
    }
}

pub fn despawn_entities<T: Component>(
    time: Res<Time>,
    mut commands: Commands,
    mut npc_query: Query<(Entity, &mut Despawns), With<T>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Check for NPCs that have received flowers and collect them for despawning
    let mut to_despawn = Vec::new();
    for (entity, mut despawns) in &mut npc_query {
        match &mut despawns.timer {
            Some(timer) => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    to_despawn.push(entity);
                }
            },
            _ => (),
        }
    }
    
    // Despawn the NPCs that have received flowers
    for entity in &to_despawn {
        commands.entity(*entity).despawn();
    }
    
    // Spawn new NPCs in random positions to replace despawned ones
    if  TypeId::of::<T>() == TypeId::of::<NpcEntity>() {
        for _ in 0..to_despawn.len() {
            spawn_npc(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                Vec3::new(
                    rand::random::<f32>() * 400.0 - 200.0,
                    rand::random::<f32>() * 400.0 - 200.0,
                    0.0
                ),
                if rand::random::<f32>() > 0.5 {
                    NpcGender::Male
                } else {
                    NpcGender::Female
                }
            );
        }
    }
}

pub fn spawn_npc(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    position: Vec3,
    gender: NpcGender,
) {
    // Load different textures for male and female NPCs
    let texture = match gender {
        NpcGender::Male => asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png"),
        NpcGender::Female => asset_server.load("textures/rpg/chars/mani/mani-idle-run.png"),
    };

    // The sprite sheet has 7 sprites arranged in a row, and they are all 24px x 24px
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // The sprite runs at 20 FPS
    let animation_config = AnimationConfig::new(1, 6, 20);

    commands.spawn((
        NpcEntity::new(gender),
        ReceivedFlowers::new(),
        Sprite {
            image: texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            }),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(6.0)).with_translation(position),
        animation_config,
        Despawns{ timer: None }
    ));
}

// Animation system for NPCs
pub fn execute_npc_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite, &NpcEntity)>,
) {
    for (mut config, mut sprite, npc) in &mut query {
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
            && npc.moving
        {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = config.first_sprite_index;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}
