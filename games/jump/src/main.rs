use std::path::Path;

use framework::wgpu;

#[derive(Debug)]
struct Jump {}

impl framework::Demo for Jump {
    async fn init(display: &framework::Display, res_dir: &Path) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    fn resize(&mut self, display: &framework::Display) {}

    fn update(&mut self, display: &framework::Display, dt: std::time::Duration) {}

    fn render(&mut self, display: &mut framework::Display) {
        let frame = display.surface().get_current_texture().unwrap();

        let view = frame.texture.create_view(&Default::default());

        let mut encoder = display.create_command_encoder(&Default::default());

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        display.queue.submit([encoder.finish()]);
        frame.present();
    }
}

fn main() {
    framework::run::<Jump>().unwrap();
}
