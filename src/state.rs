use std::num::NonZeroU32;

#[derive(Debug)]
pub struct State {
    width: u32,
    height: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_size: wgpu::Extent3d,
    output_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    pub async fn new(width: u32, height: u32) -> Self {
        let instace = wgpu::Instance::new(wgpu::Backends::all());
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        };
        let adapter = instace.request_adapter(&adapter_options).await.unwrap();
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        };

        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());
        let output_buffer = Self::create_output_buffer(&device, width, height);
        let render_pipeline = Self::create_render_pipeline(&device, texture_desc.format.clone());
        Self {
            width,
            height,
            device,
            queue,
            texture,
            output_buffer,
            render_pipeline,
            texture_view,
            texture_size: texture_desc.size,
        }
    }

    fn create_output_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);
        output_buffer
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let binding = [Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &binding,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };
        device.create_render_pipeline(&render_pipeline_desc)
    }

    pub async fn render(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let u32_size = std::mem::size_of::<u32>() as u32;
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0u64,
                    bytes_per_row: NonZeroU32::new(u32_size * self.width),
                    rows_per_image: NonZeroU32::new(self.height),
                },
            },
            self.texture_size,
        );
        self.queue.submit(Some(encoder.finish()));
        {
          let buffer_slice = self.output_buffer.slice(..);
      

          let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
          buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
              tx.send(result).unwrap();
          });
          self.device.poll(wgpu::Maintain::Wait);
          rx.receive().await.unwrap().unwrap();
      
          let data = buffer_slice.get_mapped_range();
      
          use image::{ImageBuffer, Rgba};
          let buffer =
              ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, data).unwrap();
          buffer.save("image.png").unwrap();
      
      }
      self.output_buffer.unmap();
    }
}
