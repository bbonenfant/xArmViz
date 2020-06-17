use std::ops::Range;
use wgpu::{BindGroup, Buffer};
use super::{Material, Mesh, Model};
use crate::texture::Texture;


/// Trait for a renderable Vertex.
pub trait Vertex {
    fn describe<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}


/// Trait for rendering a Model.
pub trait DrawModel<'a, 'b> where 'b: 'a {

    /// Draw an instanced Mesh to the screen.
    ///
    /// # Arguments
    ///
    /// `mesh`            - The Mesh object to be drawn.
    /// `material`        - The Material object associated with the Mesh.
    /// `uniforms`        - The Uniform objects needed for rendering, as a `wgpu::BindGroup` object.
    /// `light`           - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    /// `instances`       - A Range object indexing the instances to be rendered.
    /// `instances_buffer - The `wgpu::Buffer` objct containing the instancing data for each instance of the mesh.
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
        instances: Range<u32>,
        instance_buffer: &'b Buffer,
        shadow_bind_group: &'b wgpu::BindGroup,
    );

    /// Draw a Model to the screen.
    ///
    /// # Arguments
    ///
    /// `model`    - The Model object to be drawn.
    /// `uniforms` - The Uniform objects needed for rendering, as a `wgpu::BindGroup` object.
    /// `light`    - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    fn draw_model(&mut self, model: &'b Model, uniforms: &'b wgpu::BindGroup, light: &'b wgpu::BindGroup, shadow_bind_group: &'b wgpu::BindGroup);
}

/// Trait for rendering a Model.
pub trait ConstructShadowMap<'a, 'b> where 'b: 'a {

    /// Draw an instanced Mesh to the screen.
    ///
    /// # Arguments
    ///
    /// `mesh`            - The Mesh object to be drawn.
    /// `light`           - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    /// `instances`       - A Range object indexing the instances to be rendered.
    /// `instances_buffer - The `wgpu::Buffer` objct containing the instancing data for each instance of the mesh.
    fn construct_mesh_shadow(
        &mut self,
        mesh: &'b Mesh,
        light: &'b wgpu::BindGroup,
        instances: Range<u32>,
        instance_buffer: &'b Buffer,
    );

    /// Draw a Model to the screen.
    ///
    /// # Arguments
    ///
    /// `model`    - The Model object to be drawn.
    /// `light`    - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    fn construct_model_shadow(&mut self, model: &'b Model, light: &'b wgpu::BindGroup);
}


/// Implement Model drawing for the `wgpu::RenderPass` object.
impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a> where 'b: 'a {

    /// Draw an instanced Mesh to the screen.
    ///
    /// # Arguments
    ///
    /// `mesh`            - The Mesh object to be drawn.
    /// `material`        - The Material object associated with the Mesh.
    /// `uniforms`        - The Uniform objects needed for rendering, as a `wgpu::BindGroup` object.
    /// `light`           - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    /// `instances`       - A Range object indexing the instances to be rendered.
    /// `instances_buffer - The `wgpu::Buffer` objct containing the instancing data for each instance of the mesh.
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        uniforms: &'b BindGroup,
        light: &'b wgpu::BindGroup,
        instances: Range<u32>,
        instance_buffer: &'b Buffer,
        shadow_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
        self.set_vertex_buffer(1, instance_buffer, 0, 0);
        self.set_index_buffer(&mesh.index_buffer, 0, 0);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, &uniforms, &[]);
        self.set_bind_group(2, &light, &[]);
        self.set_bind_group(3, &shadow_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    /// Draw a Model to the screen.
    ///
    /// # Arguments
    ///
    /// `model`    - The Model object to be drawn.
    /// `uniforms` - The Uniform objects needed for rendering, as a `wgpu::BindGroup` object.
    /// `light`    - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    fn draw_model(&mut self, model: &'b Model, uniforms: &'b BindGroup, light: &'b BindGroup, shadow_bind_group: &'b wgpu::BindGroup) {
        let instances = 0..model.instances.len() as u32;
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, uniforms, light, instances.clone(), model.get_instance_buffer(), shadow_bind_group);
        }
    }
}

/// Implement Model drawing for the `wgpu::RenderPass` object.
impl<'a, 'b> ConstructShadowMap<'a, 'b> for wgpu::RenderPass<'a> where 'b: 'a {

    /// Draw an instanced Mesh to the screen.
    ///
    /// # Arguments
    ///
    /// `mesh`            - The Mesh object to be drawn.
    /// `light`           - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    /// `instances`       - A Range object indexing the instances to be rendered.
    /// `instances_buffer - The `wgpu::Buffer` objct containing the instancing data for each instance of the mesh.
    fn construct_mesh_shadow(
        &mut self,
        mesh: &'b Mesh,
        light: &'b wgpu::BindGroup,
        instances: Range<u32>,
        instance_buffer: &'b Buffer,
    ) {
        self.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
        self.set_vertex_buffer(1, instance_buffer, 0, 0);
        self.set_index_buffer(&mesh.index_buffer, 0, 0);
        self.set_bind_group(0, &light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    /// Draw a Model to the screen.
    ///
    /// # Arguments
    ///
    /// `model`    - The Model object to be drawn.
    /// `light`    - The Light object needed for rendering, as a `wgpu::BindGroup` object.
    fn construct_model_shadow(&mut self, model: &'b Model, light: &'b wgpu::BindGroup)  {
        let instances = 0..model.instances.len() as u32;
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.construct_mesh_shadow(mesh, light, instances.clone(), model.get_instance_buffer());
        }
    }
}