use std::path::Path;

use framework::{
    CameraBinder, CameraBinding,
    debug::{ColoredVertex, DebugPipeline, LineBatch},
    glam::{self, Vec2, vec2, vec3},
    wgpu,
    winit::keyboard::KeyCode,
};

mod data;
mod systems;

use data::*;
use systems::*;

/// A simple jumping game ala Doodle Jump
#[derive(Debug)]
struct Jump {
    camera: Camera,
    camera_binding: CameraBinding,
    debug: DebugPipeline,
    player: Player,
    movement_system: PlayerMovementSystem,
    bounce_system: PlayerBounceSystem,
    platforms: Vec<Platform>,
    inputs: Inputs,
}

impl framework::Demo for Jump {
    async fn init(display: &framework::Display, _res_dir: &Path) -> anyhow::Result<Self> {
        let device = &display.device;

        let camera = Camera {
            position: glam::vec2(0.0, 0.0),
            size: glam::vec2(display.config.width as f32, display.config.height as f32),
        };

        let camera_binder = CameraBinder::new(device);
        let camera_binding = camera_binder.bind(device, &camera, &camera);

        let debug = DebugPipeline::new(display, &camera_binder)?;

        // Manually spawn some platforms
        let platforms = vec![
            Platform::simple_platform(vec2(0.0, -100.0)),
            Platform::breakable_platform(vec2(0.0, 100.0)),
        ];

        Ok(Self {
            camera,
            camera_binding,
            debug,
            player: Player {
                position: vec2(0.0, 0.0),
                velocity: Vec2::ZERO,
                size: vec2(20.0, 50.0),
            },
            platforms,
            movement_system: PlayerMovementSystem::new(DEFAULT_GRAVITY),
            bounce_system: PlayerBounceSystem,
            inputs: Inputs::default(),
        })
    }

    fn resize(&mut self, display: &framework::Display) {
        self.camera.size = glam::vec2(display.config.width as _, display.config.height as _);
        self.camera_binding
            .update(&self.camera, &self.camera, &display.queue);
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
        let dt = dt.as_secs_f32();

        self.movement_system
            .run(dt, &self.inputs, &mut self.player, &mut self.camera);
        self.bounce_system
            .run(&mut self.platforms, &mut self.player);

        self.camera_binding
            .update(&self.camera, &self.camera, &display.queue);
    }

    fn render(&mut self, display: &mut framework::Display) {
        let frame = display.surface().get_current_texture().unwrap();

        let view = frame.texture.create_view(&Default::default());

        {
            self.debug.clear();
            let mut batch = self.debug.batch_lines(&display.queue);
            batch.push_box(self.player.position, self.player.size, vec3(0.0, 1.0, 1.0));

            for platform in &self.platforms {
                batch.push_box(platform.position, platform.size, vec3(1.0, 0.0, 0.0));
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
