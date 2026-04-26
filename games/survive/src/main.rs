use std::path::Path;

use anyhow::{Context, Ok};
use framework::{glam, resources::load_string};

use crate::{render::RenderController, world::World};

mod level;
mod render;
mod world;

struct Survive {
    render: RenderController,
    camera: Camera,
    camera_id: render::CameraId,
    player_sprite: framework::resources::sprite::SpriteId,
    world: World,
}

impl Survive {
    pub fn load_level(&mut self, path: impl AsRef<Path>) {
        self.world.clear();
    }
}

impl std::fmt::Debug for Survive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Survive").finish()
    }
}

impl framework::Demo for Survive {
    async fn init(display: &framework::Display, res_dir: &Path) -> anyhow::Result<Self> {
        let resources_str = load_string(res_dir.join("data/survive/resources.json")).await?;
        let resources = serde_json::from_str(&resources_str)?;

        let mut render = RenderController::new(display, &resources, res_dir).await?;

        let camera = Camera {
            position: glam::vec2(0.0, 0.0),
            size: glam::vec2(
                (display.config.width / 2) as f32,
                (display.config.height / 2) as f32,
            ),
        };

        let camera_id = render.bind_camera(display, &camera, &camera);

        let player_sprite = render.get_sprite("player").context("No player sprite")?;

        Ok(Self {
            render,
            camera,
            camera_id,
            player_sprite,
            world: World::new(),
        })
    }

    fn resize(&mut self, display: &framework::Display) {
        self.camera.size = glam::vec2(
            (display.config.width / 2) as _,
            (display.config.height / 2) as _,
        );
        self.render
            .update_camera(display, self.camera_id, &self.camera, &self.camera);
    }

    fn update(&mut self, display: &framework::Display, dt: std::time::Duration) {}

    fn render(&mut self, display: &mut framework::Display) {
        self.render
            .draw_sprite(self.player_sprite, glam::vec2(0.0, 0.0));
        self.render.flush(display, self.camera_id);
    }
}

/// 2D camera aligned to bottom left hand corner
// Maybe pixel snapping?
#[derive(Debug)]
struct Camera {
    position: glam::Vec2,
    size: glam::Vec2,
}

impl framework::Camera for Camera {
    fn calc_view(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(glam::vec3(-self.position.x, -self.position.y, 0.0))
    }
}

impl framework::Projection for Camera {
    fn calc_proj(&self) -> glam::Mat4 {
        let half = self.size * 0.5;
        glam::Mat4::orthographic_rh(-half.x, half.x, -half.y, half.y, 0.0, 1.0)
    }
}

fn main() {
    framework::run::<Survive>().unwrap();
}
