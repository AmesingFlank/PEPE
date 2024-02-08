use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    engine::{toolbox::Toolbox, value_store::ValueStore},
    ir::{ComputeRadialGradientMaskOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{Buffer, BufferProperties, RingBuffer},
    runtime::{ColorSpace, ImageFormat, ImageProperties},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ComputeRadialGradientMaskImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    ring_buffer: RingBuffer,
}
impl ComputeRadialGradientMaskImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/radial_gradient_mask.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("RadialGradientMask"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>() * 6,
                host_readable: false,
            },
        );

        ComputeRadialGradientMaskImpl {
            runtime,
            pipeline,
            bind_group_manager,
            ring_buffer,
        }
    }
}
impl ComputeRadialGradientMaskImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available()
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ComputeRadialGradientMaskOp,
        value_store: &mut ValueStore,
        toolbox: &mut Toolbox,
    ) {
        let target_img = value_store.map.get(&op.target).unwrap().as_image().clone();

        let mask_img_properties = ImageProperties {
            format: ImageFormat::Rgba8Unorm,
            ..target_img.properties
        };
        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &mask_img_properties,
        );

        let buffer = self.ring_buffer.get();

        let mask = &op.mask;

        self.runtime.queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(&[
                mask.center_x,
                mask.center_y,
                mask.radius_x,
                mask.radius_y,
                mask.feather,
                mask.rotation,
            ]),
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(buffer),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureStorage(&output_img, 0),
                },
            ],
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipeline);

            let num_workgroups_x = div_up(output_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(output_img.properties.dimensions.1, 16);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }

        toolbox
            .mipmap_generator
            .encode_mipmap_generation_command(&output_img, encoder);
    }
}
