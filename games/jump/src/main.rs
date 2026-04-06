use std::path::Path;

use anyhow::Context;
use framework::{
    CameraBinder, CameraBinding,
    debug::{ColoredVertex, DebugPipeline, LineBatch},
    glam::{self, Vec2, Vec3, vec2},
    resources::{
        load_string,
        sound::SoundSystem,
        sprite::SpritePipeline,
    },
    wgpu,
    winit::keyboard::KeyCode,
};

mod data;
mod systems;

use data::*;
use systems::*;

const RED: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const CYAN: Vec3 = Vec3::new(0.0, 1.0, 1.0);
const YELLOW: Vec3 = Vec3::new(1.0, 1.0, 0.0);
const MAGENTA: Vec3 = Vec3::new(1.0, 0.0, 1.0);

/// A simple jumping game ala Doodle Jump
struct Jump {
    camera: Camera,
    camera_binding: CameraBinding,
    debug: DebugPipeline,
    inputs: Inputs,
    player: Player,
    movement_system: PlayerMovementSystem,
    bounce_system: PlayerBounceSystem,
    platforms: Vec<Platform>,
    platform_spawn_system: PlatformSpawnSystem,
    sound_system: SoundSystem,
    sprite_pipeline: SpritePipeline,
    red_platform: framework::resources::sprite::SpriteId,
    yellow_platform: framework::resources::sprite::SpriteId,
    purple_platform: framework::resources::sprite::SpriteId,
    sprite_map: framework::resources::sprite::SpriteMapId,
    render_debug: bool,
}

impl std::fmt::Debug for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Jump { ... }").finish()
    }
}

impl framework::Demo for Jump {
    async fn init(display: &framework::Display, res_dir: &Path) -> anyhow::Result<Self> {
        let mut sound_system = SoundSystem::new()?;

        let device = &display.device;

        let camera = Camera {
            position: glam::vec2(0.0, 0.0),
            size: glam::vec2(display.config.width as f32, display.config.height as f32),
        };

        let camera_binder = CameraBinder::new(device);
        let camera_binding = camera_binder.bind(device, &camera, &camera);

        let mut sprite_pipeline =
            SpritePipeline::new(&display.device, &camera_binder, display.config.format);

        let sprite_map = sprite_pipeline
            .load_sprite_map(
                &display.device,
                &display.queue,
                res_dir.join("sprites/jump.json"),
            )
            .await?;

        let map = sprite_pipeline.get_map(sprite_map).unwrap();

        let red_platform = map
            .find_id_by_name("red_platform")
            .context("No red_platform")?;
        let purple_platform = map
            .find_id_by_name("purple_platform")
            .context("No purple_platform")?;
        let yellow_platform = map
            .find_id_by_name("yellow_platform")
            .context("No yellow_platform")?;

        let debug = DebugPipeline::new(display, &camera_binder)?;

        // Manually spawn some platforms
        let platforms = vec![Platform::simple_platform(vec2(0.0, -100.0))];

        let platform_spawn_system = PlatformSpawnSystem::new(100.0);

        let player_stats: PlayerStats = {
            let json = load_string("res/data/jump.json").await?;
            serde_json::from_str(&json)?
        };

        let bounce_system = PlayerBounceSystem::new(&mut sound_system, res_dir).await?;

        Ok(Self {
            camera,
            camera_binding,
            debug,
            inputs: Inputs::default(),
            player: Player {
                stats: player_stats,
                position: vec2(0.0, 0.0),
                velocity: Vec2::ZERO,
                size: vec2(20.0, 50.0),
            },
            platforms,
            movement_system: PlayerMovementSystem,
            bounce_system,
            platform_spawn_system,
            sound_system,
            sprite_pipeline,
            sprite_map,
            red_platform,
            yellow_platform,
            purple_platform,
            render_debug: false,
        })
    }

    fn resize(&mut self, display: &framework::Display) {
        self.camera.size = glam::vec2(display.config.width as _, display.config.height as _);
        self.camera_binding
            .update(&self.camera, &self.camera, &display.device, &display.queue);
    }

    fn handle_keyboard(&mut self, key: KeyCode, pressed: bool) {
        let f_pressed = if pressed { 1.0 } else { 0.0 };
        match (key, pressed) {
            (KeyCode::ArrowLeft | KeyCode::KeyA, _) => {
                self.inputs.left = f_pressed;
            }
            (KeyCode::ArrowRight | KeyCode::KeyD, _) => {
                self.inputs.right = f_pressed;
            }
            _ => {}
        }
    }

    fn update(&mut self, display: &framework::Display, dt: std::time::Duration) {
        self.sound_system.run();

        let dt = dt.as_secs_f32();

        self.movement_system
            .run(dt, &self.inputs, &mut self.player, &mut self.camera);
        self.bounce_system.run(
            &mut self.sound_system,
            &mut self.platforms,
            &mut self.player,
        );
        self.platform_spawn_system
            .run(&self.player, &mut self.platforms);

        self.camera_binding
            .update(&self.camera, &self.camera, &display.device, &display.queue);
    }

    fn render(&mut self, display: &mut framework::Display) {
        let frame = match display.surface().get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                display.configure();
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Outdated => return,
            wgpu::CurrentSurfaceTexture::Lost => panic!("Surface lost!"),
        };

        let view = frame.texture.create_view(&Default::default());

        {
            self.debug.clear();
            let mut batch = self.debug.batch_lines(&display.device, &display.queue);
            batch.push_box(self.player.position, self.player.size, CYAN);

            // TODO: expand this when player sprite is done
            if self.render_debug {
                for platform in &self.platforms {
                    batch.push_box(
                        platform.position,
                        platform.size,
                        if platform.bounciness > 1.0 {
                            MAGENTA
                        } else if platform.breakable {
                            RED
                        } else {
                            YELLOW
                        },
                    );
                }
            }
        }

        if let Some(mut batch) =
            self.sprite_pipeline
                .batch(&display.device, &display.queue, self.sprite_map)
        {
            for platform in &self.platforms {
                let id = if platform.breakable {
                    self.red_platform
                } else if platform.bounciness > 1.0 {
                    self.purple_platform
                } else {
                    self.yellow_platform
                };

                batch.draw_sprite(id, platform.position);
            }
        }

        let mut encoder = display.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.sprite_pipeline.draw_sprites(self.sprite_map, &mut pass, &self.camera_binding);

            self.debug.draw_lines(&mut pass, &self.camera_binding);
        }

        display.queue.submit([encoder.finish()]);
        frame.present();
    }
}

trait LineBatchExt {
    fn push_box(&mut self, position: glam::Vec2, size: glam::Vec2, color: glam::Vec3) -> &mut Self;
}

impl<'a> LineBatchExt for LineBatch<'a> {
    fn push_box(&mut self, position: glam::Vec2, size: glam::Vec2, color: glam::Vec3) -> &mut Self {
        let half = size * 0.5;
        let top_right = position + half;
        let bot_left = position - half;
        let points = [
            ColoredVertex {
                position: glam::vec3(bot_left.x, bot_left.y, 0.0),
                color,
            },
            ColoredVertex {
                position: glam::vec3(top_right.x, bot_left.y, 0.0),
                color,
            },
            ColoredVertex {
                position: glam::vec3(top_right.x, top_right.y, 0.0),
                color,
            },
            ColoredVertex {
                position: glam::vec3(bot_left.x, top_right.y, 0.0),
                color,
            },
        ];

        self.push(points[0], points[1])
            .push(points[1], points[2])
            .push(points[2], points[3])
            .push(points[3], points[0])
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
    framework::run::<Jump>().unwrap();
}
