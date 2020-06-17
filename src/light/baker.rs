use crate::{model::Model, state::StateCore};
use super::Light;

pub const BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor = {
    const VISIBILITY: wgpu::ShaderStage = wgpu::ShaderStage::from_bits_truncate(
        wgpu::ShaderStage::VERTEX.bits() | wgpu::ShaderStage::FRAGMENT.bits()
    );
    wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: VISIBILITY,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: None,
    }
};

pub struct ShadowBaker {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub views: Vec<wgpu::TextureView>,
}

impl ShadowBaker {
    pub fn new(core: &StateCore, views: Vec<wgpu::TextureView>) -> Self {
        
        let buffer = core.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Single Light Shadow Buffer"),
                size: super::LightRaw::SIZE,
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST
            }
        );

        let bind_group_layout =
            core.device.create_bind_group_layout(&BIND_GROUP_LAYOUT_DESC);

        let bind_group =
            core.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &buffer,
                                range: 0..super::LightRaw::SIZE,
                            },
                        }
                    ],
                    label: None,
                }
            );
        
        let render_pipeline =
            create_render_pipeline(
                &core, 
                &[&bind_group_layout],
                &crate::shaders::SHADOW_SHADER_DATA
            );
        
        return ShadowBaker { buffer, bind_group, bind_group_layout, render_pipeline, views }
    }
    
    pub fn bake_shadows(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        light: &Light,
        view_index: usize,
        models: &Vec<Model>
    ) {
        self.copy_into_buffer(light.get_buffer(), encoder);
        let mut render_pass = 
            encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    color_attachments: &[],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.views[view_index],
                            depth_load_op: wgpu::LoadOp::Clear,
                            depth_store_op: wgpu::StoreOp::Store,
                            clear_depth: 1.0,
                            stencil_load_op: wgpu::LoadOp::Clear,
                            stencil_store_op: wgpu::StoreOp::Store,
                            clear_stencil: 0,
                        }
                    ),
                }
            );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        for model in models.iter() {
            let instances = 0..model.instances.len() as u32;
            let instance_buffer = model.get_instance_buffer();
            for mesh in model.meshes.iter() {
                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                render_pass.set_vertex_buffer(1, instance_buffer, 0, 0);
                render_pass.set_index_buffer(&mesh.index_buffer, 0, 0);
                render_pass.draw_indexed(0..mesh.num_elements, 0, instances.clone());
            }
        }
    }

    fn copy_into_buffer(&self, buffer: &wgpu::Buffer, encoder: &mut wgpu::CommandEncoder) {
        const SOURCE_OFFSET: wgpu::BufferAddress = 0;
        const DESTINATION_OFFSET: wgpu::BufferAddress = 0;
        const COPY_SIZE: wgpu::BufferAddress = super::LightRaw::SIZE;
        encoder.copy_buffer_to_buffer(
            &buffer,
            SOURCE_OFFSET,
            &self.buffer,
            DESTINATION_OFFSET,
            COPY_SIZE
        );
    }
}

fn create_render_pipeline(
    core: &StateCore,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader_data: &crate::shaders::ShaderData
) -> wgpu::RenderPipeline {

    let module: wgpu::ShaderModule;
    let fragment_stage = if let Some(ref data) = shader_data.fragment {
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
                    depth_bias: 2,
                    depth_bias_slope_scale: 2.0,
                    depth_bias_clamp: 0.0,
                }
            ),
            color_states: &[],
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