use cgmath::{Matrix4, Vector3};
use wgpu::BufferAddress;


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

    __padding: f32,

    pub view_projection: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Zeroable for LightRaw {}
unsafe impl bytemuck::Pod for LightRaw {}

impl LightRaw {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as BufferAddress;
    const PADDING: f32 = 0.0;

    pub fn new(position: Vector3<f32>, color: Vector3<f32>, view_projection: Matrix4<f32>) -> Self {
        return LightRaw{ 
            position, 
            _padding: Self::PADDING,
            color,
            __padding: Self::PADDING,
            view_projection,
        }
    }

    // pub fn size_of(&self) -> wgpu::BufferAddress {
    //     return std::mem::size_of_val(self) as BufferAddress
    // }
}