use cgmath::{Point3, Vector3};
use wgpu::Device;

use crate::{
    camera::{Projection, View},
    state::StateCore
};
use super::{LightSource, LightRaw};


/// Structure for holding information about the light source 
///   that is sent to the Shader programs.
pub struct Spotlight {

    // The Bind Group used for rendering.
    pub bind_group: wgpu::BindGroup,
    
    // The Buffer used to send data to the GPU.
    buffer: wgpu::Buffer,

    // The RGB value for the color of the light.
    color: cgmath::Vector3<f32>,

    view: View,

    projection: Projection,

    view_projection: cgmath::Matrix4<f32>,
}

impl Spotlight {

    const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
    const RED: [f32; 3] = [0.75, 0.0, 0.0];

    /// Creates a new Light object.
    ///
    /// # Arguments
    ///
    /// * `device`   - The connection to the graphics device. Used to create the rendering resources.
    /// * `position` - The 3D position of the light source.
    /// * `color`    - The RGB value for the color of the light.
    pub fn new(device: &Device, color: Vector3<f32>, projection: Projection, view: View, bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let view_projection = projection.as_matrix() * view.as_matrix();

        let light_raw = {
            use cgmath::EuclideanSpace;
            LightRaw::new(view.get_position().to_vec(), color, view_projection)
        };

        let buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[light_raw]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
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
                            range: 0..LightRaw::SIZE,
                        },
                    },
                ],
                label: None,
            }
        );
        
        return Spotlight{ color, bind_group, buffer, view, projection, view_projection }
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
    pub fn get_position(&self) -> Point3<f32> { self.view.get_position() }

    /// Set the color of the Light object.
    ///
    /// # Arguments
    ///
    /// * `position` - The new 3D position of the light source.
    /// * `core`     - Structure for holding the WGPU primitives for running a windowed application.
    pub fn set_position(&mut self, position: Point3<f32>, core: &StateCore) {
        self.view.set_position(position);
        self.view_projection = self.projection.as_matrix() * self.view.as_matrix();
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
        let light_raw = {
            use cgmath::EuclideanSpace;
            LightRaw::new(self.view.get_position().to_vec(), self.color, self.view_projection)
        };
        let staging_buffer = core.device.create_buffer_with_data(
            bytemuck::cast_slice(&[light_raw]), 
            wgpu::BufferUsage::COPY_SRC
        );

        // Copy the data from the staging buffer into the Light buffer.
        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, LightRaw::SIZE);
        core.submit(&[encoder.finish()]);
    }
}

impl LightSource for Spotlight {

    fn as_light_raw(&self) -> LightRaw {
        use cgmath::EuclideanSpace;
        LightRaw::new(self.view.get_position().to_vec(), self.color, self.view_projection)
    }

    fn get_buffer(&self) -> &wgpu::Buffer { &self.buffer }
    fn get_bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }
}