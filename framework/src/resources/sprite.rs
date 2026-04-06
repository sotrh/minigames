use std::path::{Path, PathBuf};

use glam::vec2;

use crate::{
    resources::{load_binary, load_string},
    texture_from_rgba_img, CameraBinder, CameraBinding, RawBuffer,
};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SpriteId(usize);

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SpriteMapId(usize);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Sprite {
    name: String,
    min: glam::Vec2,
    max: glam::Vec2,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SpriteMap {
    name: String,
    version: String,
    texture: String,
    dimensions: glam::Vec2,
    sprites: Vec<Sprite>,
}

impl SpriteMap {
    pub async fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let sprites: SpriteMap = serde_json::from_str(&load_string(path).await?)?;
        Ok(sprites)
    }

    pub fn find_id_by_name(&self, name: &str) -> Option<SpriteId> {
        self.sprites
            .iter()
            .enumerate()
            .filter(|&(_, sprite)| sprite.name == name)
            .next()
            .map(|(i, _)| SpriteId(i))
    }

    fn get(&self, id: SpriteId) -> Option<&Sprite> {
        self.sprites.get(id.0)
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpriteVertex {
    pos: glam::Vec2,
    uv: glam::Vec2,
}

impl SpriteVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as _,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
    };
}

pub struct SpritePipeline {
    sprite_maps: Vec<SpriteMap>,
    bind_groups: Vec<wgpu::BindGroup>,
    buffers: Vec<SpriteBuffer>,
    pixel_sampler: wgpu::Sampler,
    draw_sprites: wgpu::RenderPipeline,
    sprite_layout: wgpu::BindGroupLayout,
}

impl SpritePipeline {
    pub fn new(
        device: &wgpu::Device,
        camera_binder: &CameraBinder,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let sprite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SpritePipeline::sprite_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pixel_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SpritePipeline::pixel_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SpritePipeline::pipeline_layout"),
            bind_group_layouts: &[Some(camera_binder.layout()), Some(&sprite_layout)],
            immediate_size: 0,
        });

        let sprite_shader = device.create_shader_module(wgpu::include_wgsl!("sprite.wgsl"));

        let draw_sprites = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SpritePipeline::draw_sprites"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sprite_shader,
                entry_point: Some("position_sprite"),
                compilation_options: Default::default(),
                buffers: &[SpriteVertex::LAYOUT],
            },
            primitive: wgpu::PrimitiveState {
                // cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sprite_shader,
                entry_point: Some("texture_sprite"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            sprite_layout,
            sprite_maps: Vec::new(),
            bind_groups: Vec::new(),
            buffers: Vec::new(),
            pixel_sampler,
            draw_sprites,
        }
    }

    pub async fn load_sprite_map(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<SpriteMapId> {
        let path = path.as_ref();

        let dir = match path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => PathBuf::new(),
        };

        let sprite_map = SpriteMap::load(path).await?;

        let img = image::load_from_memory(&load_binary(dir.join(&sprite_map.texture)).await?)?;
        let texture = texture_from_rgba_img(device, queue, &img, true);
        let texture_view = texture.create_view(&Default::default());

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SpritePipeline::sprite_bg"),
            layout: &self.sprite_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.pixel_sampler),
                },
            ],
        });

        let id = SpriteMapId(self.sprite_maps.len());

        self.sprite_maps.push(sprite_map);
        self.bind_groups.push(bg);
        self.buffers.push(SpriteBuffer::with_capacity(device, 128));

        Ok(id)
    }

    pub fn draw_sprites(
        &self,
        map_id: SpriteMapId,
        pass: &mut wgpu::RenderPass<'_>,
        camera_binding: &CameraBinding,
    ) {
        if map_id.0 > self.sprite_maps.len() {
            return;
        }

        // TODO: maybe just bundle these together with sprite_maps?
        let bg = &self.bind_groups[map_id.0];
        let buffer = &self.buffers[map_id.0];

        let num_indices = buffer.indices.data.len() as u32;

        pass.set_pipeline(&self.draw_sprites);
        pass.set_bind_group(0, camera_binding.bind_group(), &[]);
        pass.set_bind_group(1, bg, &[]);
        pass.set_vertex_buffer(0, buffer.vertices.buffer.slice(..));
        pass.set_index_buffer(buffer.indices.buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    pub fn get_map(&self, sprite_map: SpriteMapId) -> Option<&SpriteMap> {
        self.sprite_maps.get(sprite_map.0)
    }

    pub fn batch<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        sprite_map: SpriteMapId,
    ) -> Option<SpriteBatch<'a>> {
        match (
            self.buffers.get_mut(sprite_map.0),
            self.sprite_maps.get(sprite_map.0),
        ) {
            (Some(buffer), Some(sprites)) => {
                buffer.clear();
                Some(buffer.batch(device, queue, sprites))
            },
            _ => None,
        }
    }
}

pub struct SpriteBuffer {
    vertices: RawBuffer<SpriteVertex>,
    indices: RawBuffer<u32>,
}

impl SpriteBuffer {
    pub fn with_capacity(device: &wgpu::Device, capacity: usize) -> Self {
        let vertices = RawBuffer::with_capacity(
            device,
            capacity * 4,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );
        let indices = RawBuffer::with_capacity(
            device,
            capacity * 6,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        );

        Self { vertices, indices }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn batch<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        sprites: &'a SpriteMap,
    ) -> SpriteBatch<'a> {
        SpriteBatch {
            dirty: false,
            buffer: self,
            sprites,
            device,
            queue,
        }
    }
}

pub struct SpriteBatch<'a> {
    dirty: bool,
    buffer: &'a mut SpriteBuffer,
    sprites: &'a SpriteMap,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
}

impl<'a> SpriteBatch<'a> {
    pub fn draw_sprite(&mut self, id: SpriteId, position: glam::Vec2) -> &mut Self {
        if let Some(sprite) = self.sprites.get(id) {
            self.dirty = true;

            let size = sprite.max - sprite.min;

            let min = position - size * 0.5;
            let max = position + size * 0.5;

            let mut min_uv = sprite.min / self.sprites.dimensions;
            let mut max_uv = sprite.max / self.sprites.dimensions;

            // TODO: maybe have this configurable
            std::mem::swap(&mut min_uv.y, &mut max_uv.y);

            let start_index = self.buffer.vertices.data.len() as u32;

            self.buffer.vertices.data.push(SpriteVertex {
                pos: vec2(min.x, min.y),
                uv: vec2(min_uv.x, min_uv.y),
            });
            self.buffer.vertices.data.push(SpriteVertex {
                pos: vec2(max.x, min.y),
                uv: vec2(max_uv.x, min_uv.y),
            });
            self.buffer.vertices.data.push(SpriteVertex {
                pos: vec2(max.x, max.y),
                uv: vec2(max_uv.x, max_uv.y),
            });
            self.buffer.vertices.data.push(SpriteVertex {
                pos: vec2(min.x, max.y),
                uv: vec2(min_uv.x, max_uv.y),
            });

            self.buffer.indices.data.push(start_index + 0);
            self.buffer.indices.data.push(start_index + 1);
            self.buffer.indices.data.push(start_index + 2);
            self.buffer.indices.data.push(start_index + 0);
            self.buffer.indices.data.push(start_index + 2);
            self.buffer.indices.data.push(start_index + 3);
        }
        self
    }
}

impl<'a> Drop for SpriteBatch<'a> {
    fn drop(&mut self) {
        if self.dirty {
            self.buffer
                .vertices
                .update(&self.device, &self.queue, |_| {});
            self.buffer
                .indices
                .update(&self.device, &self.queue, |_| {});
        }
    }
}
