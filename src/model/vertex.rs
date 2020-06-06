use wgpu::{BufferAddress, VertexBufferDescriptor};
use super::Vertex;

/// Describes a single vertex of a Model.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    
    // The 3D position of the vertex.
    pub position: [f32; 3],

    // The coordinates for texture mapping.
    pub tex_coords: [f32; 2],

    // The normal vector.
    pub normal: [f32; 3],
}

/// Used for serializing the ModelVertex structure.
unsafe impl bytemuck::Pod for ModelVertex {}
unsafe impl bytemuck::Zeroable for ModelVertex {}

/// Constants describing the location in memory of the items in the structure.
impl ModelVertex {
    pub const SIZE: BufferAddress = std::mem::size_of::<Self>() as BufferAddress;
    pub const POSITION_OFFSET: BufferAddress = 0  as BufferAddress;
    pub const TEX_COORDS_OFFSET: BufferAddress = std::mem::size_of::<[f32; 3]>() as BufferAddress;
    pub const NORMAL_OFFSET: BufferAddress = 
        Self::TEX_COORDS_OFFSET + (std::mem::size_of::<[f32; 2]>() as BufferAddress);
}

impl Vertex for ModelVertex {

    /// Creates a `wgpu::VertexBufferDecriptor` that describes the `ModelVertex` struct.
    fn describe<'a>() -> VertexBufferDescriptor<'a> {
        return VertexBufferDescriptor {
            stride: Self::SIZE,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: Self::POSITION_OFFSET,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::TEX_COORDS_OFFSET,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::NORMAL_OFFSET,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
            ]
        }
    }
}