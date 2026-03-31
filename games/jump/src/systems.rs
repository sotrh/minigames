use std::path::Path;

use framework::{
    glam::vec2,
    math::{self, Box2},
    rand::{self, Rng, random, thread_rng},
    resources::sound::{SoundOptions, SoundSystem},
};

use crate::{
    Camera,
    data::{Inputs, Platform, Player},
};

#[derive(Debug)]
pub struct PlayerMovementSystem;

impl PlayerMovementSystem {
    pub fn run(&self, dt: f32, inputs: &Inputs, player: &mut Player, camera: &mut Camera) {
        // Instant movement left and right
        player.velocity.x = (inputs.right - inputs.left) * player.stats.move_speed;

        // Double gravity when moving down
        let gravity_mod = if player.velocity.y > 0.0 { 1.0 } else { 2.0 };

        player.velocity += vec2(0.0, player.stats.gravity * gravity_mod * dt);
        player.position += player.velocity * dt;

        // For now just lock the camera to the player
        camera.position = player.position;
    }
}

#[derive(Debug)]
pub struct PlayerBounceSystem {
    high_jump: framework::resources::sound::TrackId,
    jump: framework::resources::sound::TrackId,
}

impl PlayerBounceSystem {
    pub async fn new(sound_system: &mut SoundSystem, res_dir: &Path) -> anyhow::Result<Self> {
        let high_jump = sound_system
            .load(res_dir.join("sound/high-jump.mp3"))
            .await?;
        let jump = sound_system.load(res_dir.join("sound/jump.mp3")).await?;

        Ok(Self { high_jump, jump })
    }

    pub fn run(
        &self,
        sound_system: &mut SoundSystem,
        platforms: &mut Vec<Platform>,
        player: &mut Player,
    ) {
        if player.velocity.y > 0.0 {
            return;
        }

        let player_bbox = Box2::with_position_size(player.position, player.size);

        let mut max_bounciness = 0.0;
        let mut hits = Vec::new();
        for (i, platform) in platforms.iter().enumerate() {
            let pbb = Box2::with_position_size(platform.position, platform.size);

            if math::intersect_box2_box2(&player_bbox, &pbb) {
                hits.push(i);

                if max_bounciness < platform.bounciness {
                    max_bounciness = platform.bounciness;
                }
            }
        }

        if !hits.is_empty() {
            player.velocity.y = player.stats.jump_speed * max_bounciness;

            let mut rng = thread_rng();
            let pitch_shift = rng.gen_range(-0.1..=0.1);
            if max_bounciness > 1.0 {
                sound_system.play(self.high_jump, SoundOptions { pitch_shift });
            } else {
                sound_system.play(self.jump, SoundOptions { pitch_shift });
            }
        }

        for hit in hits.into_iter().rev() {
            if platforms[hit].breakable {
                platforms.swap_remove(hit);
            }
        }
    }
}

#[derive(Debug)]
pub struct PlatformSpawnSystem {
    current_y: f32,
}

impl PlatformSpawnSystem {
    const PLATFORM_DISTANCE: f32 = 200.0;
    const SPAWN_DISTANCE: f32 = Self::PLATFORM_DISTANCE * 10.0;
    const DESPAWN_DISTANCE: f32 = Self::SPAWN_DISTANCE;

    pub fn new(start_y: f32) -> Self {
        Self { current_y: start_y }
    }

    pub fn run(&mut self, player: &Player, platforms: &mut Vec<Platform>) {
        // Despawn platforms that are to far down
        let mut to_remove = Vec::new();
        for (i, platform) in platforms.iter().rev().enumerate() {
            if self.current_y - platform.position.y > Self::DESPAWN_DISTANCE {
                to_remove.push(i);
            }
        }

        // Spawn new platforms if player is high enough
        if player.position.y + Self::SPAWN_DISTANCE > self.current_y {
            let mut rng = thread_rng();

            let num_platforms = rng.gen_range(1..=3);

            let (platform_spacing, platform_offset) = match num_platforms {
                1 => (0.0, 0.0),
                2 => (200.0, -100.0),
                _ => (200.0, -200.0),
            };

            for i in 0..num_platforms {
                let x = i as f32 * platform_spacing + platform_offset;
                let position = vec2(x, self.current_y);

                let r: f32 = rng.r#gen();

                if r < 0.6 {
                    platforms.push(Platform::simple_platform(position));
                } else if r < 0.9 {
                    platforms.push(Platform::breakable_platform(position));
                } else {
                    platforms.push(Platform::bouncy_platform(position));
                }
            }

            self.current_y += Self::PLATFORM_DISTANCE;
        }
    }
}
