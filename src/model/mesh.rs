/// Describes a 3D Mesh and the associated components needed for rendering.
pub struct Mesh {

    // An identifying name.
    pub name: String,

    // The Buffer of vertices.
    pub vertex_buffer: wgpu::Buffer,

    // The Buffer for indices.
    pub index_buffer: wgpu::Buffer,

    // The number of elements in the mesh.
    pub num_elements: u32,

    // The index of the Material for the Mesh.
    // This is used for lookup in the Model's vector of Materials.
    pub material: usize,
}