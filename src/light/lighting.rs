use std::collections::hash_map::{HashMap, Keys, Values, ValuesMut};
use cgmath::Vector3;
use wgpu::{BindGroup, Color, Device, RenderPass};

use crate::{
    camera::{Projection, View},
    model::{Instance, Model},
    shaders::ShaderData,
    state::StateCore,
};
use super::{Light, LightSource, ShadowBaker, Spotlight};


pub struct Lighting {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub full_bind_group_layout: wgpu::BindGroupLayout,

    lights: HashMap<String, Light>,

    lights_bind_group: wgpu::BindGroup,

    count_buffer: wgpu::Buffer,
    lights_buffer: wgpu::Buffer,

    render_pipeline: wgpu::RenderPipeline,

    shadow_baker: super::ShadowBaker,
    pub shadow_texture: crate::texture::Texture,
}

type ctp = u32;

impl Lighting {
    pub const MAX_LIGHTS: usize = 10;

    pub fn new(core: &StateCore, uniforms_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let full_bind_group_layout = 
            core.device.create_bind_group_layout(&super::BIND_GROUP_LAYOUT_DESC);
        let bind_group_layout = 
            core.device.create_bind_group_layout(&super::baker::BIND_GROUP_LAYOUT_DESC);
        
        let render_pipeline = {
            // These BindGroupLayouts define the structure of the data that will be sent to GPU
            //    and used during the shader programs.
            let bind_group_layouts = &[
                &uniforms_bind_group_layout,
                &bind_group_layout,
            ];

            // Construct the render pipeline (the pipeline for sending data to the GPU and executing
            //   the shader programs).
            create_render_pipeline(&core, bind_group_layouts, &crate::shaders::LIGHT_SHADER_DATA)
        };

        let count_buffer = core.device.create_buffer_with_data(
            bytemuck::cast_slice(&[0 as ctp]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let lights_buffer_size = 
            (Self::MAX_LIGHTS as wgpu::BufferAddress) * super::LightRaw::SIZE;
        let lights_buffer = core.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Uniform Buffer -- All Lights"),
                size: lights_buffer_size,
                usage: (
                      wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::UNIFORM
                ),
            }
        );
        let lights_bind_group = core.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Bind Group -- All Lights"),
                layout: &full_bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &count_buffer,
                            range: 0..(std::mem::size_of::<ctp>() as wgpu::BufferAddress),
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &lights_buffer,
                            range: 0..lights_buffer_size,
                        },
                    }
                ],    
            }
        );

        let shadow_texture = 
            crate::texture::Texture::create_shadow_texture(&core.device, &core.swap_chain_desc, "Shadow Texture");
        let shadow_baker = {            
            let shadow_views: Vec<wgpu::TextureView> = 
                (0..Self::MAX_LIGHTS).map(|index| {
                    shadow_texture.texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            format: crate::texture::Texture::DEPTH_FORMAT,
                            dimension: wgpu::TextureViewDimension::D2,
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: index as u32,
                            array_layer_count: 1,
                        }
                    )
                }
            ).collect();
            ShadowBaker::new(&core, shadow_views)
        };
        
        let lights = HashMap::with_capacity(Self::MAX_LIGHTS);
        return Lighting { bind_group_layout, full_bind_group_layout, lights, lights_bind_group, count_buffer, lights_buffer, render_pipeline, shadow_baker, shadow_texture }
    }

    pub fn add_spotlight(
        &mut self,
        device: &Device,
        name: String,
        color: Color,
        projection: Projection,
        view: View,
    ) -> Result<wgpu::CommandBuffer, ()> {
        let color = Vector3::new(color.r as f32, color.g as f32, color.b as f32);
        let spotlight = Spotlight::new(device, color, projection, view, &self.bind_group_layout);

        // Move the instance of the light box to the position of the Light object.
        let light_model = {
            use cgmath::EuclideanSpace;
            let mut model = Model::new_light(device).unwrap(); 
            let instance = Instance::from_position(spotlight.get_position().to_vec());
            model.set_instances(vec![instance], &device);
            model
        };

        let light = Light::new(spotlight, light_model);

        let light_index = self.insert::<Spotlight>(name.clone(), light)?;
        let light = self.lights.get(&name).unwrap();

        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("update encoder") }
        );

        // Copy the data from the staging buffer into the Light buffer.
        let destination_offset = (light_index as wgpu::BufferAddress) * super::LightRaw::SIZE;
        encoder.copy_buffer_to_buffer(&light.get_buffer(), 0, &self.lights_buffer, destination_offset, super::LightRaw::SIZE);

        let new_count_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[1 + light_index as ctp]),
            wgpu::BufferUsage::COPY_SRC,
        );
        encoder.copy_buffer_to_buffer(&new_count_buffer, 0, &self.count_buffer, 0, std::mem::size_of::<ctp>() as wgpu::BufferAddress);

        Ok(encoder.finish())
    }

    pub fn bake(&self, encoder: &mut wgpu::CommandEncoder, models: &Vec<Model>) {
        for (index, light) in self.lights.values().enumerate() {
            self.shadow_baker.bake_shadows(encoder, light, index, models);            
        }
    }

    pub fn render<'r>(&'r self, render_pass: &mut RenderPass<'r>, uniforms_bind_group: &'r BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        self.lights
            .values()
            .for_each(|light| {
                if light.visible {
                    let model = light.get_model();
                    let instances = 0..model.instances.len() as u32;
                    render_pass.set_bind_group(0, &uniforms_bind_group, &[]);
                    render_pass.set_bind_group(1, &light.get_bind_group(), &[]);
                    for mesh in &model.meshes {
                        render_pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                        render_pass.set_vertex_buffer(1, model.get_instance_buffer(), 0, 0);
                        render_pass.set_index_buffer(&mesh.index_buffer, 0, 0);
                        render_pass.draw_indexed(0..mesh.num_elements, 0, instances.clone());
                    }
                }
            })
    }

    pub fn get_lights_buffer(&self) -> &wgpu::Buffer { &self.lights_buffer }
    pub fn get_bind_group(&self) -> &wgpu::BindGroup { &self.lights_bind_group }

    pub fn get(&self, name: &str) -> Option<&Light> { self.lights.get(name) }
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Light> { self.lights.get_mut(name) }
    pub fn keys(&self) -> Keys<'_, String, Light> { self.lights.keys() }
    pub fn values(&self) -> Values<'_, String, Light> { self.lights.values() }
    pub fn values_mut(&mut self) -> ValuesMut<'_, String, Light> { self.lights.values_mut() }


    fn insert<T>(&mut self, key: String, value: Light) -> Result<usize, ()>
      where T: LightSource + 'static {
        let light_count = self.lights.len(); 
        if light_count == (Self::MAX_LIGHTS - 1) {
            return Err(())
        }
        self.lights.insert(key, value);
        Ok(light_count)
    }
}


/// Create a new RenderPipeline object.
fn create_render_pipeline<'p>(
    core: &StateCore,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader_data: &ShaderData
) -> wgpu::RenderPipeline {

    let module: wgpu::ShaderModule;
    let fragment_stage = 
          if let Some(ref data) = shader_data.fragment {
            module = core.device.create_shader_module(&data);
            Some(wgpu::ProgrammableStageDescriptor { module: &module, entry_point: "main" })
        } else { None };

    let vertex_stage = wgpu::ProgrammableStageDescriptor { 
        module: &core.device.create_shader_module(&shader_data.vertex), 
        entry_point: "main"
    };
    let render_pipeline_layout = core.device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor { bind_group_layouts }
    );

    return core.device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: vertex_stage,
            fragment_stage: fragment_stage,
            rasterization_state: Some(
                wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }
            ),
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: core.swap_chain_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: Some(
                wgpu::DepthStencilStateDescriptor {
                    format: crate::texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_read_mask: 0,
                    stencil_write_mask: 0,
                }
            ),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: {
                    use crate::model::{InstanceRaw, ModelVertex, Vertex};
                    &[ModelVertex::describe(), InstanceRaw::describe()]
                },
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        }
    )
}