use bevy::{math::Vec2, prelude::*};
use std::time::Duration;

#[derive(Component)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(
            Duration::from_secs_f32(1.0 / (fps as f32)),
            TimerMode::Repeating,
        )
    }
}

#[derive(Component)]
pub struct LinearMotion {
    pub speed: Vec2,
}

pub fn move_linear_motion(time: Res<Time>, mut query: Query<(&mut Transform, &LinearMotion)>) {
    let dt = time.delta_secs();
    for (mut transform, linear_motion) in &mut query {
        let translation_2d = transform.translation.truncate() + linear_motion.speed * dt;
        transform.translation = Vec3::from_array([translation_2d.x, translation_2d.y, 0.0]);
    }
}
