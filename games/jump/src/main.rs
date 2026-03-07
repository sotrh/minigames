use std::path::Path;

use framework::{
    CameraBinder, CameraBinding,
    debug::{ColoredVertex, DebugPipeline, LineBatch},
    glam, wgpu,
};

/// A simple jumping game ala Doodle Jump
#[derive(Debug)]
struct Jump {
    camera: Camera,
    camera_binding: CameraBinding,
    debug: DebugPipeline,
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

        let mut debug = DebugPipeline::new(display, &camera_binder)?;

        {
            let a = ColoredVertex {
                position: glam::vec3(10.0, 10.0, 0.0),
                color: glam::vec3(1.0, 0.0, 0.0),
            };
            let b = ColoredVertex {
                position: glam::vec3(110.0, 10.0, 0.0),
                color: glam::vec3(0.0, 1.0, 0.0),
            };
            let c = ColoredVertex {
                position: glam::vec3(110.0, 110.0, 0.0),
                color: glam::vec3(0.0, 0.0, 1.0),
            };

            debug
                .batch_lines(&display.queue)
                .push(a, b)
                .push(b, c)
                .push(c, a)
                .push_box(
                    glam::vec2(200.0, 200.0),
                    glam::vec2(50.0, 50.0),
                    glam::vec3(0.0, 1.0, 1.0),
                );
        }

        Ok(Self {
            camera,
            camera_binding,
            debug,
        })
    }

    fn resize(&mut self, display: &framework::Display) {
        self.camera.size = glam::vec2(display.config.width as _, display.config.height as _);
        self.camera_binding
            .update(&self.camera, &self.camera, &display.queue);
    }

    fn update(&mut self, _display: &framework::Display, _dt: std::time::Duration) {}

    fn render(&mut self, display: &mut framework::Display) {
        let frame = display.surface().get_current_texture().unwrap();

        let view = frame.texture.create_view(&Default::default());

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
        let top_right = position + size;
        let points = [
            ColoredVertex {
                position: glam::vec3(position.x, position.y, 0.0),
                color,
            },
            ColoredVertex {
                position: glam::vec3(top_right.x, position.y, 0.0),
                color,
            },
            ColoredVertex {
                position: glam::vec3(top_right.x, top_right.y, 0.0),
                color,
            },
            ColoredVertex {
                position: glam::vec3(position.x, top_right.y, 0.0),
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
        glam::Mat4::from_translation(glam::vec3(self.position.x, self.position.y, 0.0))
    }
}

impl framework::Projection for Camera {
    fn calc_proj(&self) -> glam::Mat4 {
        glam::Mat4::orthographic_rh(0.0, self.size.x, 0.0, self.size.y, 0.0, 1.0)
    }
}

fn main() {
    framework::run::<Jump>().unwrap();
}
