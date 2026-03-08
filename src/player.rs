use bevy::math::Vec2;
// use bevy_ecs::prelude::*;
use bevy::prelude::*;

const RUN_SPEED: f32 = 300.0;

#[derive(Component)]
pub struct PlayerEntity {
    pub speed: f32,
    pub run_direction: Vec2,
    pub running: bool,
}


impl PlayerEntity {
    pub fn new() -> Self {
        Self {
            speed: RUN_SPEED,
            run_direction: Vec2::new(0.0,0.0),
            running: false,
        }
    }

    pub fn set_movement_speed(keyboard: Res<ButtonInput<KeyCode>>,mut query: Query<&mut Self>) {
        for mut player in &mut query {
            player.run_direction.x = 0.0;
            player.run_direction.y = 0.0;
            if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA])
            {
                player.run_direction.x += -1.0;
            }
            if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD])
            {
                player.run_direction.x += 1.0;
            }
            if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW])
            {
                player.run_direction.y += 1.0;
            }
            if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS])
            {
                player.run_direction.y += -1.0;
            }
            player.running = player.run_direction.length() > 0.1;
            player.run_direction = player.run_direction.normalize_or_zero();
        }
    }
}