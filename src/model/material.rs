/// Decribes the Testure and the associated components for rendering.
pub struct Material {

    // An identifying name for the material.
    pub name: String,

    // The Texture object.
    pub diffuse_texture: crate::texture::Texture,

    // The bind group used for rendering.
    pub bind_group: wgpu::BindGroup,
}