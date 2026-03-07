use glam::Vec3;

use crate::{CameraBinder, CameraBinding, Display, RawBuffer};

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ColoredVertex {
    pub position: Vec3,
    pub color: Vec3,
}

impl ColoredVertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as _,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
        ],
    };
}

#[derive(Debug)]
pub struct DebugPipeline {
    lines_pipeline: wgpu::RenderPipeline,
    lines_buffer: RawBuffer<ColoredVertex>,
}

impl DebugPipeline {
    pub fn new(display: &Display, camera_binder: &CameraBinder) -> anyhow::Result<Self> {
        let device = &display.device;

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lines_pipeline::layout"),
            bind_group_layouts: &[camera_binder.layout()],
            immediate_size: 0,
        });

        let module = device.create_shader_module(wgpu::include_wgsl!("debug.wgsl"));

        let lines_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("DebugPipeline::lines_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[ColoredVertex::LAYOUT],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("draw_colored"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: display.config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let lines_buffer = RawBuffer::with_capacity(
            device,
            128,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Ok(Self {
            lines_pipeline,
            lines_buffer,
        })
    }

    pub fn batch_lines<'a>(&'a mut self, queue: &'a wgpu::Queue) -> LineBatch<'a> {
        LineBatch {
            pipeline: self,
            queue,
            dirty: false,
        }
    }

    pub fn draw_lines(&self, pass: &mut wgpu::RenderPass<'_>, binding: &CameraBinding) {
        pass.set_pipeline(&self.lines_pipeline);
        pass.set_vertex_buffer(0, self.lines_buffer.buffer.slice(..));
        pass.set_bind_group(0, binding.bind_group(), &[]);
        pass.draw(0..self.lines_buffer.data.len() as _, 0..1);
    }
}

pub struct LineBatch<'a> {
    pipeline: &'a mut DebugPipeline,
    queue: &'a wgpu::Queue,
    dirty: bool,
}

impl<'a> LineBatch<'a> {
    pub fn push(&mut self, a: ColoredVertex, b: ColoredVertex) -> &mut Self {
        self.pipeline.lines_buffer.data.push(a);
        self.pipeline.lines_buffer.data.push(b);
        self.dirty = true;
        self
    }
}

impl<'a> Drop for LineBatch<'a> {
    fn drop(&mut self) {
        if self.dirty {
            self.pipeline.lines_buffer.update(&self.queue, |_| {});
        }
    }
}
