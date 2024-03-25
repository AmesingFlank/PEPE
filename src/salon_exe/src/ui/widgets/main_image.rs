use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap, num::NonZeroU64};

use eframe::egui::Ui;
use eframe::egui_wgpu::ScreenDescriptor;
use eframe::{egui, egui_wgpu};
use salon_core::runtime::Image;
use salon_core::runtime::Sampler;
use salon_core::runtime::{
    BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager, BindingResource,
    Runtime,
};
use salon_core::runtime::{Buffer, BufferProperties, RingBuffer};
use salon_core::shader::{Shader, ShaderLibraryModule};
use salon_core::utils::rectangle::Rectangle;

use crate::ui::get_ui_crop_rect;
use crate::ui::utils::get_max_image_size;

pub struct MainImageCallback {
    pub image: Arc<Image>,
    pub rotation_degrees: Option<f32>,
    pub crop_rect: Option<Rectangle>,
    pub mask: Option<Arc<Image>>,
    pub ui_max_rect: egui::Rect,
}

impl MainImageCallback {
    pub fn required_allocated_rect(&self) -> egui::Rect {
        // TODO: rewrite MainImageCallback so that it occupies the entire ui
        let full_image_ui_rect_size = get_max_image_size(
            self.image.aspect_ratio(),
            self.ui_max_rect.width(),
            self.ui_max_rect.height(),
        );
        let full_image_ui_rect =
            egui::Rect::from_center_size(self.ui_max_rect.center(), full_image_ui_rect_size);
        full_image_ui_rect
    }

    pub fn cropped_image_ui_rect(&self) -> egui::Rect {
        let full_image_ui_rect_size = get_max_image_size(
            self.image.aspect_ratio(),
            self.ui_max_rect.width(),
            self.ui_max_rect.height(),
        );
        let full_image_ui_rect =
            egui::Rect::from_center_size(self.ui_max_rect.center(), full_image_ui_rect_size);
        get_ui_crop_rect(
            full_image_ui_rect,
            self.crop_rect.clone().unwrap_or(Rectangle::regular()),
        )
    }
}

impl egui_wgpu::CallbackTrait for MainImageCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut resources: &mut MainImageRenderResources = resources.get_mut().unwrap();
        resources.prepare(device, queue, self);
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &MainImageRenderResources = resources.get().unwrap();
        resources.paint(render_pass, self.image.as_ref());
    }
}

pub struct MainImageRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<u32, BindGroupDescriptorKey>, // image uuid -> key
    ring_buffer: RingBuffer,
    texture_sampler: Sampler,
}

impl MainImageRenderResources {
    pub fn new(runtime: Arc<Runtime>, target_format: wgpu::TextureFormat) -> Self {
        let shader_code = Shader::from_code(include_str!("../shaders/main_image.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_render_pipeline(shader_code.as_str(), target_format, Some("MainImage"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout)
            .with_label("MainImage".to_owned());

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<u32>() * 2 + 5 * size_of::<f32>(),
                host_readable: false,
            },
        );

        let texture_sampler = runtime.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        MainImageRenderResources {
            pipeline,
            bind_group_manager,
            bind_group_key_cache: HashMap::new(),
            ring_buffer,
            texture_sampler,
        }
    }

    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_call: &MainImageCallback,
    ) {
        let buffer = self.ring_buffer.get();
        queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(&[
                render_call.image.properties.color_space as u32,
                render_call.mask.is_some() as u32,
            ]),
        );
        let rotation_degrees = render_call.rotation_degrees.clone().unwrap_or(0.0);
        let rotation_radians = rotation_degrees.to_radians();
        let crop_rect = render_call
            .crop_rect
            .clone()
            .unwrap_or(Rectangle::regular());
        queue.write_buffer(
            &buffer.buffer,
            size_of::<u32>() as u64 * 2,
            bytemuck::cast_slice(&[
                rotation_radians,
                crop_rect.center.x,
                crop_rect.center.y,
                crop_rect.size.x,
                crop_rect.size.y,
            ]),
        );

        let bind_group_desc = BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(buffer),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Texture(&render_call.image),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.texture_sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: if let Some(ref m) = render_call.mask {
                        BindingResource::Texture(&m)
                    } else {
                        BindingResource::Texture(&render_call.image)
                    },
                },
            ],
        };
        let bind_group_key = bind_group_desc.to_key();
        self.bind_group_manager.ensure(bind_group_desc);
        self.bind_group_key_cache
            .insert(render_call.image.uuid, bind_group_key);
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, image: &'rp Image) {
        let bind_group_key = self.bind_group_key_cache.get(&image.uuid).unwrap();
        let bind_group = self
            .bind_group_manager
            .get_from_key_or_panic(bind_group_key);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
