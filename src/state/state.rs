use futures::executor::block_on;
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, Color, Device};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    camera::{Camera, CameraController, Projection, View},
    light::Lighting,
    model::{Instance, Model},
    shaders::{ShaderData, LIGHT_SHADER_DATA, MODEL_SHADER_DATA, SHADOW_SHADER_DATA},
    texture::Texture,
    Uniforms,
};
use super::{Renderer, StateCore};

const TEXTURE_BIND_GROUP_LAYOUT_DESC: BindGroupLayoutDescriptor = 
    BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Float,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
            },
        ],
        label: Some("Texture Bind Group Layout"),
    };

const SHADOW_BIND_GROUP_LAYOUT_DESC: BindGroupLayoutDescriptor = 
    BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Float,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
            },
        ],
        label: Some("Shadow Bind Group Layout"),
    };


/// The State of the Application.
pub struct State {
    
    // Structure for holding the WGPU primitives for running a windowed application.
    core: StateCore,

    // The renderer object of the Models.
    model_renderer: Renderer,

    lighting: Lighting,

    // The Camera object, i.e. the Viewer.
    camera: Camera,

    // The controller of the Camer object. This processes events to affect the position of the Camera.
    camera_controller: CameraController,

    // The Uniform (constant) objects that get sent to the GPU.
    uniforms: Uniforms,

    // The texture object used to track the pixel depth of rendered objects (from the Camera's perspective).
    depth_texture: Texture,

    shadow_bind_group: wgpu::BindGroup,
}

impl State {

    /// Construct a new State from a `winit::window::Window` object.
    pub fn new(window: &Window) -> Self {

        // The core of the State object.
        let core: StateCore = block_on(StateCore::new(window));

        // The Camera and Camera Controller objects.
        let camera = Camera::new(
            View::default(),
            Projection::with_aspect( core.get_aspect_ratio()),
        );
        let camera_controller = CameraController::new();

        // Uniforms.
        let uniforms = Uniforms::new(
            &core.device, 
            camera.get_view().get_position().to_homogeneous(),
            camera.build_view_projection_matrix(),
        );

        // Create the Light object. (This is point from which light shines, not the physical light box).
        use cgmath::Deg;
        let mut lighting = Lighting::new(&core, &uniforms.bind_group_layout);
        let cmd = lighting.add_spotlight(
            &core.device,
            String::from("Spotlight 1"),
            Color::WHITE,
            Projection::new(
                core.get_aspect_ratio(), 
                Deg(135.0),
                Projection::DEFAULT_Z_NEAR,
                Projection::DEFAULT_Z_FAR
            ),
            View::new(
                (8.0, 12.0, 0.0).into(),
                (0.0, 0.0, 0.0).into(),
                (0.0, 1.0, 0.0).into()
            ),
        ).unwrap();
        core.submit(&[cmd]);
        let cmd = lighting.add_spotlight(
            &core.device,
            String::from("Spotlight 2"),
            Color::WHITE,
            Projection::new(
                core.get_aspect_ratio(), 
                Deg(135.0),
                Projection::DEFAULT_Z_NEAR,
                Projection::DEFAULT_Z_FAR
            ),
            View::new(
                (-6.0, 15.0, 0.0).into(),
                (0.0, 0.0, 0.0).into(),
                (0.0, 1.0, 0.0).into()
            ),
        ).unwrap();
        core.submit(&[cmd]);

        // Texture Bind Group Layout.
        let texture_bind_group_layout = 
            core.device.create_bind_group_layout(&TEXTURE_BIND_GROUP_LAYOUT_DESC);
        let shadow_bind_group_layout =
            core.device.create_bind_group_layout(&SHADOW_BIND_GROUP_LAYOUT_DESC);
        
        
        
        // Render Pipelines.
        let model_renderer = {
            // Create the model object and submit them to the GPU.
            let (mut obj_model, cmds) = 
                Model::load(&core.device, &texture_bind_group_layout, "src/res/sphere.obj").unwrap();
            core.submit(&cmds);
            
            // Construct the instances of these objects (if they need to be replicated).
            let instances = create_tutorial_instances();
            obj_model.set_instances(instances, &core.device);

            let (floor_model, cmds) = create_floor(&core.device, &texture_bind_group_layout);
            core.submit(&cmds);

            // These BindGroupLayouts define the structure of the data that will be sent to GPU
            //    and used during the shader programs.
            let bind_group_layouts = &[
                &texture_bind_group_layout,
                &uniforms.bind_group_layout,
                &lighting.full_bind_group_layout,
                &shadow_bind_group_layout,
            ];

            // Construct the render pipeline (the pipeline for sending data to the GPU and executing
            //   the shader programs).
            let render_pipeline = create_render_pipeline(
                &core, bind_group_layouts, &MODEL_SHADER_DATA
            );

            let shadow_pipeline = create_render_pipeline(
                &core, &[&lighting.bind_group_layout], &SHADOW_SHADER_DATA
            );

            Renderer::new(vec![obj_model, floor_model], render_pipeline, Some(shadow_pipeline))
        };

        // Depth Texture.
        let depth_texture = 
            Texture::create_depth_texture(&core.device, &core.swap_chain_desc, "Depth Texture");

        let shadow_bind_group = core.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &shadow_bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&lighting.shadow_texture.view)
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&lighting.shadow_texture.sampler)
                    },
                ],
                label: None,
            }
        );

        return Self {
            core,
            model_renderer,
            camera,
            camera_controller,
            lighting,
            uniforms,
            depth_texture,
            shadow_bind_group,
        }
    }

    /// Handle a resizing of the window.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.core.resize(new_size);
        self.depth_texture = 
            Texture::create_depth_texture(&self.core.device, &self.core.swap_chain_desc, "Depth Texture");
    }

    /// Handle the Window events.
    ///
    /// This includes processing events on the camera controller (see for more information),
    /// And processing the following events here:
    ///   * If the `L` key is pressed, toggle the visibility of the light box.
    ///
    /// # Returns
    /// 
    /// Boolean of whether an event was handled.
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let handled_event = self.camera_controller.process_events(event);
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput { state, virtual_keycode: Some(keycode), .. },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::L => { 
                        self.lighting
                            .values_mut()
                            .for_each(|light| {
                                light.visible ^= is_pressed;
                            }
                        ); 
                    },
                    _ => return handled_event,
                }
            },
            _ => return handled_event,
        }
        return true
    }

    /// Make updates to the scene and data being sent to the GPU.
    pub fn update(&mut self) {
        // use cgmath::{EuclideanSpace, Point3};

        // // Move the camera in a circular motion.
        // let new_position = {
        //     use cgmath::{Deg, Quaternion};
        //     use cgmath::Rotation3;

        //     let old_position = self.light.get_position();
        //     Point3::from_vec(Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Deg(1.0)) * old_position.to_vec())
        // };

        // self.light.set_position(new_position, &self.core);
        // let light_instance = Instance::from_position(self.light.get_position().to_vec());
        // self.light_renderer.models[0].set_instances(vec![light_instance], &self.core.device);

        // Make updates to the camera and uniform objects if necessary.
        if self.camera_controller.update_camera(&mut self.camera) {
            self.uniforms.update_from_camera(&self.camera, &self.core);
        }
    }

    /// Render the scene.
    pub fn render(&mut self) {

        let frame = 
            self.core.swap_chain.get_next_texture().expect("Timeout getting texture");

        let mut encoder = self.core.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );
        self.lighting.bake(&mut encoder, &self.model_renderer.models);
        
        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
                    }
                ],
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &self.depth_texture.view,
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
        self.model_renderer.render(&mut render_pass, &self.uniforms, &self.lighting, &self.shadow_bind_group);
        self.lighting.render(&mut render_pass, &self.uniforms.bind_group);
        drop(render_pass);
    
        self.core.submit(&[encoder.finish()]);
    }
}

fn create_floor(device: &Device, layout: &BindGroupLayout) -> (Model, Vec<wgpu::CommandBuffer>) {
    let (mut floor_model, cmds) = 
        Model::load(device, layout, "src/res/tile.obj").unwrap();
    
    const NUM_TILES: usize = 20;
    const SPACE_BETWEEN: f32 = 2.0;

    let mut instances: Vec<Instance> = 
        (0..NUM_TILES).flat_map(|z| {
            (0..NUM_TILES).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_TILES as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_TILES as f32 / 2.0);
                let position = cgmath::Vector3 { x, y: -5.0, z };
                Instance::from_position(position)
            })
        }).collect();
    
    let flip = {
        use cgmath::{Deg, Quaternion};
        use cgmath::Rotation3;
        Quaternion::from_angle_z(Deg(180.0))
    };
    let flipped_instances: Vec<Instance> = 
        instances.iter().map(|instance| {
            Instance { position: instance.position, rotation: flip }
        }).collect();
    instances.extend(flipped_instances);
    floor_model.set_instances(instances, device);
    return (floor_model, cmds)
}

/// Create a new RenderPipeline object.
fn create_render_pipeline(
    core: &StateCore,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader_data: &ShaderData
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
                    format: Texture::DEPTH_FORMAT,
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

fn create_tutorial_instances() -> Vec<Instance> {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;

    use cgmath::{InnerSpace, One, Rotation3, Zero};
    
    return
    (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
            let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
            let position = cgmath::Vector3 { x, y: 0.0, z };
    
            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not create correctly
                cgmath::Quaternion::one()
            } else {
                cgmath::Quaternion::from_axis_angle(position.clone().normalize(), cgmath::Deg(45.0))
            };
    
            Instance { position, rotation }
        })
    }).collect();
}
