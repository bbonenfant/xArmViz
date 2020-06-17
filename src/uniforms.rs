use cgmath::{Matrix4, Vector4};
use wgpu::{BufferAddress, BindGroupLayoutDescriptor, Device};
use crate::{camera::Camera, state::StateCore};

pub const BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> = {
    const VISIBILITY: wgpu::ShaderStage = wgpu::ShaderStage::from_bits_truncate(
        wgpu::ShaderStage::VERTEX.bits() | wgpu::ShaderStage::FRAGMENT.bits()
    );
    BindGroupLayoutDescriptor {
        bindings: &[ 
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: VISIBILITY,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
         ],
        label: Some("Uniform Bind Group Layout"),
    }
};

/// Structure for holding the Uniform objects that are sent to the Shader programs.
pub struct Uniforms {

    // The Bind Group used for rendering.
    pub bind_group: wgpu::BindGroup,

    // The Layout used for the Uniforms BindGroup.
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    // The Buffer used to send data to the GPU.
    buffer: wgpu::Buffer,

    // The position vector of the Viewer.
    view_position: cgmath::Vector4<f32>,

    // The View-Projection Matrix.
    view_projection: cgmath::Matrix4<f32>,
}

impl Uniforms {

    /// Create a new Uniform object.
    pub fn new(device: &Device, view_position: Vector4<f32>, view_projection: Matrix4<f32>) -> Self {
        // Create the UniformRaw object and stor it in a Buffer.
        let uniforms_raw = UniformsRaw { view_position, view_projection };
        let buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms_raw]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Create the BindGroup object for the Uniforms.
        let mem_size = std::mem::size_of_val(&uniforms_raw) as BufferAddress;
        let bind_group_layout = 
            device.create_bind_group_layout(&BIND_GROUP_LAYOUT_DESC);
        let bind_group = 
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    bindings: &[
                        wgpu::Binding { 
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer { buffer: &buffer, range: 0..mem_size },
                        },
                    ],
                    label: Some("Uniforms Bind Group"),
                }
            );

        Self { bind_group, bind_group_layout, buffer, view_position, view_projection }
    }

    pub fn update_from_camera(&mut self, camera: &Camera, core: &StateCore) {
        self.view_position = camera.get_view().get_position().to_homogeneous();
        self.view_projection = camera.build_view_projection_matrix();
        self.update_buffer(core);
    }

    /// Set the View-Projection matrix.
    #[allow(dead_code)]
    pub fn set_view_projection(&mut self, matrix: Matrix4<f32>, core: &StateCore) {
        self.view_projection = matrix;
        self.update_buffer(core);
    }

    /// Update the buffer of UniformsRaw objects that is sent to the GPU.
    ///
    /// # Arguments
    ///
    /// * `core` - Structure for holding the WGPU primitives for running a windowed application.
    fn update_buffer(&mut self, core: &StateCore) {
        // A Command encoder is used to perform Copy operations on the GPU.
        let mut encoder = core.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("update encoder") }
        );

        // Create a staging buffer with the updated Buffer data.
        let uniforms_raw = UniformsRaw { view_position: self.view_position, view_projection: self.view_projection };
        let staging_buffer = core.device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms_raw]), 
            wgpu::BufferUsage::COPY_SRC
        );

        // Copy the data from the staging buffer into the Uniforms buffer.
        let copy_size = std::mem::size_of_val(&uniforms_raw) as wgpu::BufferAddress;
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, copy_size);
        core.submit(&[encoder.finish()]);
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct UniformsRaw {
    // The position vector of the Viewer.
    view_position: cgmath::Vector4<f32>,
    // The View-Projection Matrix.
    view_projection: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for UniformsRaw {}
unsafe impl bytemuck::Zeroable for UniformsRaw {}
