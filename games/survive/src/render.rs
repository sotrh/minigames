use std::path::Path;

use framework::{
    CameraBinder, CameraBinding, Display, debug::DebugPipeline, glam, resources::sprite::{SpriteId, SpriteMapId, SpritePipeline}, wgpu
};

#[derive(Debug, Clone, Copy)]
pub struct CameraId(usize);

pub struct SpriteDraw(SpriteId, glam::Vec2);

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RenderResources {
    sprite_map: String,
}

pub struct RenderController {
    sprites: Vec<SpriteDraw>,
    sprite_map: SpriteMapId,
    sprite_pipeline: SpritePipeline,
    cameras: Vec<CameraBinding>,
    camera_binder: CameraBinder,
    debug: DebugPipeline,
}

impl RenderController {
    pub async fn new(
        display: &framework::Display,
        resources: &RenderResources,
        res_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let res_dir = res_dir.as_ref();

        let camera_binder = CameraBinder::new(&display.device);

        let mut sprite_pipeline =
            SpritePipeline::new(&display.device, &camera_binder, display.config.format);

        let sprite_map = sprite_pipeline
            .load_sprite_map(
                &display.device,
                &display.queue,
                dbg!(res_dir.join(&resources.sprite_map)),
            )
            .await?;

        let debug = DebugPipeline::new(display, &camera_binder)?;

        Ok(Self {
            sprite_map,
            sprite_pipeline,
            camera_binder,
            debug,
            cameras: Default::default(),
            sprites: Default::default(),
        })
    }

    pub fn flush(&mut self, display: &mut Display, camera_id: CameraId) {
        {
            let mut batch = self.sprite_pipeline.batch(&display.device, &display.queue, self.sprite_map).unwrap();
            for SpriteDraw(sprite, position) in self.sprites.drain(..) {
                batch.draw_sprite(sprite, position);
            }
        }

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

        let camera = self.cameras.get(camera_id.0).unwrap();

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

            self.sprite_pipeline
                .draw_sprites(self.sprite_map, &mut pass, camera);

            self.debug.draw_lines(&mut pass, camera);
        }

        display.queue.submit([encoder.finish()]);
        frame.present();
    }

    pub fn bind_camera(
        &mut self,
        display: &Display,
        camera: &impl framework::Camera,
        projection: &impl framework::Projection,
    ) -> CameraId {
        let id = CameraId(self.cameras.len());
        self.cameras
            .push(self.camera_binder.bind(&display.device, camera, projection));
        id
    }

    pub(crate) fn update_camera(
        &mut self,
        display: &Display,
        id: CameraId,
        camera: &impl framework::Camera,
        projection: &impl framework::Projection,
    ) {
        let binding = self.cameras.get_mut(id.0).unwrap();
        binding.update(camera, projection, &display.device, &display.queue);
    }

    pub(crate) fn get_sprite(&self, sprite_name: &str) -> Option<SpriteId> {
        self.sprite_pipeline
            .get_map(self.sprite_map)?
            .find_id_by_name(sprite_name)
    }

    pub(crate) fn draw_sprite(&mut self, player_sprite: SpriteId, position: framework::glam::Vec2) {
        self.sprites.push(SpriteDraw(player_sprite, position));
    }
}
