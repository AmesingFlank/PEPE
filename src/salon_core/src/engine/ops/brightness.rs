use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    engine::{value_store::ValueStore, shader::{ShaderLibraryFunctions, ShaderLibrary}},
    image::Image,
    ir::{BrightnessAdjust, Id, Value},
    runtime::Runtime,
};

pub struct BrightnessAdjustImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
}
impl BrightnessAdjustImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code =
            ShaderLibrary::get_library_functions_code(ShaderLibraryFunctions::ColorSpaces)
                .to_owned()
                + include_str!("./brightness.wgsl");

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());

        let uniform_buffer = runtime.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<f32>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        BrightnessAdjustImpl {
            runtime,
            pipeline,
            bind_group_layout,
            uniform_buffer,
        }
    }
}
impl BrightnessAdjustImpl {
    pub fn apply(&mut self, op: &BrightnessAdjust, value_store: &mut ValueStore) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        value_store.ensure_value_at_id_is_image_of_dimensions(
            self.runtime.as_ref(),
            op.result,
            input_img.dimensions,
        );

        let output_img = value_store.map.get(&op.result).unwrap().as_image();

        self.runtime.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[op.brightness]),
        );

        let bind_group = self
            .runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_img.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &output_img.texture_view_base_mip,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);

            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(input_img.dimensions.0, input_img.dimensions.1, 1);
        }
        self.runtime
            .encode_mipmap_generation_command(output_img.as_ref(), &mut encoder);
        self.runtime.queue.submit(Some(encoder.finish()));
    }
}