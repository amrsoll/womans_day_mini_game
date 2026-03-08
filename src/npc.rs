use bevy::math::Vec2;
use bevy::prelude::*;
use std::time::Duration;

// mod animation;
use crate::animation::*;

const NPC_SPEED: f32 = 100.0;

#[derive(Component)]
pub struct NpcEntity {
    pub speed: f32,
    pub move_direction: Vec2,
    pub moving: bool,
    pub gender: NpcGender,
}

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
}

impl ReceivedFlowers {
    pub fn new() -> Self {
        Self { has_received: false }
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
            transform.translation = Vec3::from_array([translation_2d.x, translation_2d.y, 0.0]);
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
    ));
}

// Animation system for NPCs
fn execute_animations(
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