use framework::{
    glam::vec2,
    math::{self, Box2},
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
            player.velocity.y = player.stats.jump_speed * max_bounciness;
        }

        for hit in hits.into_iter().rev() {
            if platforms[hit].breakable {
                platforms.swap_remove(hit);
            }
        }
    }
}
