use std::num::NonZeroU32;

use crate::object::Object;

#[derive(Debug)]
pub struct State {
    width: u32,
    height: u32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_size: wgpu::Extent3d,
    output_buffer: wgpu::Buffer,
    objects: Vec<Object>,
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
        Self {
            width,
            height,
            device,
            queue,
            texture,
            output_buffer,
            texture_view,
            texture_size: texture_desc.size,
            objects: vec![],
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

    pub fn set_objects(&mut self, objects: Vec<Object>) {
        self.objects = objects;
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
            let count = self.objects.len();
            let mut index = 0usize;
            while index < count {
                let obj = self.objects.get(index).unwrap();
                render_pass.set_pipeline(obj.render_pipeline.as_ref().unwrap());
                render_pass.set_bind_group(0, obj.bind_group.as_ref().unwrap(), &[]);
                render_pass.set_vertex_buffer(0, obj.vertex_buffer.as_ref().unwrap().slice(..));
                render_pass.set_index_buffer(
                    obj.index_buffer.as_ref().unwrap().slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..obj.num_indices, 0, 0..1);
                index += 1;
            }
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
