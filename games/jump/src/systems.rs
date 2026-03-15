use framework::{glam::vec2, math::{self, Box2}};

use crate::{Camera, data::{Platform, Player}};

pub const DEFAULT_GRAVITY: f32 = -200.0;
pub const BOUNCE_VELOCITY: f32 = 300.0;

#[derive(Debug)]
pub struct PlayerMovementSystem {
    gravity: f32,
}

impl PlayerMovementSystem {

    pub fn new(gravity: f32) -> Self {
        Self { gravity }
    }

    pub fn run(&self, dt: f32, player: &mut Player, camera: &mut Camera) {
        player.velocity += vec2(0.0, self.gravity * dt);
        player.position += player.velocity * dt;

        // For now just lock the camera to the player
        camera.position = player.position;
    }
}

#[derive(Debug)]
pub struct PlayerBounceSystem;

impl PlayerBounceSystem {
    pub fn run(&self, platforms: &mut Vec<Platform>, player: &mut Player) {
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
            player.velocity.y = BOUNCE_VELOCITY * max_bounciness;
        }

        for hit in hits.into_iter().rev() {
            if platforms[hit].breakable {
                platforms.swap_remove(hit);
            }
        }
    }
}