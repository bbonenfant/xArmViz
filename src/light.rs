use cgmath::Vector3;
use wgpu::Device;

use crate::state::StateCore;


/// Structure for holding information about the light source 
///   that is sent to the Shader programs.
pub struct Light {

    // The Bind Group used for rendering.
    pub bind_group: wgpu::BindGroup,

    // The Layout used for the Uniforms BindGroup.
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    // The Buffer used to send data to the GPU.
    buffer: wgpu::Buffer,

    // The RGB value for the color of the light.
    color: cgmath::Vector3<f32>,

    // The 3D Position of the light source.
    position: cgmath::Vector3<f32>,
}

impl Light {

    const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

    /// Creates a new Light object.
    ///
    /// # Arguments
    ///
    /// * `device`   - The connection to the graphics device. Used to create the rendering resources.
    /// * `position` - The 3D position of the light source.
    /// * `color`    - The RGB value for the color of the light.
    pub fn new(device: &Device, position: Vector3<f32>, color: Vector3<f32>) -> Self {
        let light_raw = LightRaw::new(position, color);
        let light_raw_size = std::mem::size_of_val(&light_raw) as wgpu::BufferAddress;

        let buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[light_raw]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let bind_group_layout =
            device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    bindings: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                        },
                    ],
                    label: None,
            }
        );

        let bind_group = 
            device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &buffer,
                            range: 0..light_raw_size,
                        },
                    },
                ],
                label: None,
            }
        );
        
        return Light{ position, color, bind_group, bind_group_layout, buffer }
    }

    /// Creates a new white Light object.
    ///
    /// # Arguments
    ///
    /// * `device`   - The connection to the graphics device. Used to create the rendering resources.
    /// * `position` - The 3D position of the light source.
    pub fn new_white(device: &Device, position: Vector3<f32>) -> Self {
        return Self::new(device, position, Self::WHITE.into())
    }

    /// Get the color of the Light object.
    pub fn get_color(&self) -> Vector3<f32> { self.color }

    /// Set the color of the Light object.
    ///
    /// # Arguments
    ///
    /// * `color` - The RGB value for the new color of the light.
    /// * `core`  - Structure for holding the WGPU primitives for running a windowed application.
    pub fn set_color(&mut self, color: Vector3<f32>, core: &StateCore) {
        self.color = color;
        self.update_buffer(core)
    }

    /// Get the position of the Light object.
    pub fn get_position(&self) -> Vector3<f32> { self.position }

    /// Set the color of the Light object.
    ///
    /// # Arguments
    ///
    /// * `position` - The new 3D position of the light source.
    /// * `core`     - Structure for holding the WGPU primitives for running a windowed application.
    pub fn set_position(&mut self, position: Vector3<f32>, core: &StateCore) {
        self.position = position;
        self.update_buffer(core)
    }

    /// Update the buffer of LightRaw objects that is sent to the GPU.
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
        let light_raw = LightRaw::new(self.position, self.color);
        let staging_buffer = core.device.create_buffer_with_data(
            bytemuck::cast_slice(&[light_raw]), 
            wgpu::BufferUsage::COPY_SRC
        );

        // Copy the data from the staging buffer into the Light buffer.
        let copy_size = std::mem::size_of_val(&light_raw) as wgpu::BufferAddress;
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, copy_size);
        core.submit(&[encoder.finish()]);
    }
}


/// The Raw Light structure that is sent to the GPU.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightRaw {

    // The Vector representing the 3D position of the light source.
    pub position: cgmath::Vector3<f32>,

    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field.
    _padding: f32,

    // The RGB value for the color of the light.
    pub color: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for LightRaw {}
unsafe impl bytemuck::Pod for LightRaw {}

impl LightRaw {
    const PADDING: f32 = 0.0;
    pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> Self {
        return LightRaw{ position, _padding: Self::PADDING, color }
    }
}