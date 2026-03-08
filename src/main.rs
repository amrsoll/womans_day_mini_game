use bevy::{input::*, prelude::*};
use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use crate::common_conditions::input_just_pressed;

#[derive(Component)]
struct ScoreDisplay;

mod player;
use player::*;
mod npc;
use npc::*;
mod animation;
use animation::*;

// mod map;
// use map::*;

const NUMBER_NPCS: u8 = 5;

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
// enum GameState {
//     #[default]
//     Playing,
//     GameOver,
// }

#[derive(Resource, Default)]
struct Game {
    score: i32,
}

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite, &PlayerEntity)>) {
    for (mut config, mut sprite, player) in &mut query {
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
            && player.running
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

fn move_players(time: Res<Time>, mut query: Query<(&mut Transform, &PlayerEntity)>) {
    let dt = time.delta_secs();
    for (mut transform, player) in &mut query {
        
        let translation_2d = transform.translation.truncate() + player.run_direction * player.speed * dt;
        transform.translation = Vec3::from_array([translation_2d.x, translation_2d.y, 0.0]);
    }

}

fn give_flowers_to_npcs(
    mut npc_query: Query<(&mut ReceivedFlowers, &Transform, &NpcEntity)>,
    player_query: Query<(&Transform, &PlayerEntity)>,
    mut game: ResMut<Game>,
) {
    // Check if any player is close to any NPC
    for (player_transform, _player_entity) in &player_query {
        for (mut received_flowers, npc_transform, npc_entity) in &mut npc_query {
            // Calculate distance between player and NPC
            let distance = player_transform.translation.distance(npc_transform.translation);
            // If player is close to NPC (within 100 units), give flower
            if distance < 100.0 {
                received_flowers.has_received = true;
                // Increase score if female NPC receives flower
                if npc_entity.gender == NpcGender::Female {
                    game.score += 1;
                }
            }
        }
    }
}

fn update_score_display(
    game: Res<Game>,
    mut score_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    for mut text in &mut score_query {
        text.0 = format!("Score: {}", game.score);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game: ResMut<Game>
) {

    // reset the game state
    game.score = 0;

    commands.spawn(Camera2d);

    // Create a minimal UI explaining how to interact with the example
    commands.spawn((
        Text::new("WASD / Arrows for movement\nMouse to aim\nSpace to give flowers to nearby people"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

    // Display current score in bottom right corner
    commands.spawn((
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            right: px(12),
            ..default()
        },
        ScoreDisplay,
    ));

    // Load the sprite sheet using the `AssetServer`
    let texture_1 = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");

    // The sprite sheet has 7 sprites arranged in a row, and they are all 24px x 24px
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // The first (left-hand) sprite runs at 20 FPS
    let animation_config_1 = AnimationConfig::new(1, 6, 20);

    // Create the first (left-hand) sprite
    commands.spawn((
        PlayerEntity::new() ,
        Sprite {
            image: texture_1.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config_1.first_sprite_index,
            }),
            ..default()
        },
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-70.0, 0.0, 0.0)),
        animation_config_1,
    ));

    // The second (right-hand) sprite runs at 20 FPS
    let animation_config_2 = AnimationConfig::new(1, 6, 20);

    // Load the sprite sheet using the `AssetServer`
    let texture_2 = asset_server.load("textures/rpg/chars/mani/mani-idle-run.png");

    // Create the second (right-hand) sprite
    commands.spawn((
        Sprite {
            image: texture_2.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config_2.first_sprite_index,
            }),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(70.0, 0.0, 0.0)),
        animation_config_2,
        player::PlayerEntity::new(),
    ));

    // Spawn some random NPCs
    for i in 0..NUMBER_NPCS {
        let x = (i as f32 * 100.0) - 200.0;
        let y = (i as f32 * 100.0) - 200.0;
        let gender = if rand::random::<f32>() > 0.5 {
            NpcGender::Male
        } else {
            NpcGender::Female
        };
        
        spawn_npc(
            &mut commands,
            &asset_server,
            &mut texture_atlas_layouts,
            Vec3::new(x, y, 0.0),
            gender,
        );
    }
}

fn main() {
    App::new()
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(
            PreUpdate,
            (|mut mode: ResMut<DebugPickingMode>| {
                *mode = match *mode {
                    DebugPickingMode::Disabled => DebugPickingMode::Normal,
                    _ => DebugPickingMode::Disabled,
                };
            })
            .distributive_run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F3,
            )),
        )
        .init_resource::<Game>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, execute_animations)
        .add_systems(Update, npc::execute_npc_animations)
        .add_systems(Update, (PlayerEntity::set_movement_speed, move_players).chain())
        .add_systems(Update, (set_npc_movement, move_npcs).chain())
        .add_systems(Update, give_flowers_to_npcs.run_if(input_just_pressed(KeyCode::Space)))
        .add_systems(Update, update_score_display)
        .run();
}