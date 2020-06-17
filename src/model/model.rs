use std::path::Path;
use wgpu::{BindGroupLayout, BindingResource, BufferUsage, Device};
use crate::texture::Texture;
use super::{Instance, InstanceRaw, Material, Mesh, ModelVertex};


type ModelResult = Result<(Model, Vec<wgpu::CommandBuffer>), failure::Error>;

/// Describes the 3D objects to be rendered.
/// Each object that is rendered is 
pub struct Model {

    // The meshes that make up the model.
    pub meshes: Vec<Mesh>,

    // The materials used by the meshes.
    pub materials: Vec<Material>,

    // The instances of the Model to be rendered.
    pub instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

impl Model {

    /// Load the `.obj` file and all corresponding textures into a `Model` object.
    ///
    /// # Arguments
    ///
    /// * `device` - The connection to the graphics device. Used to create the rendering resources.
    /// * `layout` - The `wgpu::BindGroupLayout` object corresponding to the textures bind group.
    /// * `path`   - The path to the `.obj` file. The corresponding texture files are assumed
    ///                to be in the same directory as the `.obj` file.
    pub fn load<P: AsRef<Path>>(device: &Device, layout: &BindGroupLayout, path: P) -> ModelResult {
        // Parse the `.obj` file. Optional is enabled to triangulate mesh.
        let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), true)?;

        // We're assuming that the texture files are stored with the `.obj` file.
        let containing_folder = path.as_ref().parent().unwrap();

        // Iterate over the `tobj::Material` objects and convert them into 
        //    `crate::model::Material` objects with corresponding `wgpu::CommandBuffer` objects.
        let mut command_buffers = Vec::new();
        let mut materials = Vec::new();

        let mut texture_results = Vec::new();
        for material in obj_materials {
            let path = containing_folder.join( material.diffuse_texture);
            if let Ok(texture_result) = Texture::load(&device, path) {
                texture_results.push(texture_result);
            } else {
                texture_results.push(Texture::from_color(device, [255, 255, 255, 255].into()).unwrap());
            }
        }

        for (diffuse_texture, command_buffer) in texture_results {
            let bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: BindingResource::TextureView(&diffuse_texture.view)
                        },
                        wgpu::Binding {
                            binding: 1,
                            resource: BindingResource::Sampler(&diffuse_texture.sampler)
                        },
                    ],
                    label: None,
                }
            );
            command_buffers.push(command_buffer);
            materials.push( Material { name: String::from("name"), diffuse_texture, bind_group } )
        }

        // Iterate over the `tobj::Model` objects and convert them into `crate::model::Mesh` objects.
        let meshes: Vec<Mesh> = obj_models.into_iter()
            .map(|model| {
                let num_coords = model.mesh.positions.len() / 3;
                let vertices: Vec<ModelVertex> = (0..num_coords)
                    .map(|index| {
                        ModelVertex {
                            position: [
                                model.mesh.positions[index * 3],
                                model.mesh.positions[index * 3 + 1],
                                model.mesh.positions[index * 3 + 2],
                            ],
                            tex_coords: [
                                model.mesh.texcoords[index * 2],
                                model.mesh.texcoords[index * 2 + 1]
                            ],
                            normal: [
                                model.mesh.normals[index * 3],
                                model.mesh.normals[index * 3 + 1],
                                model.mesh.normals[index * 3 + 2],
                            ],
                        }
                    }).collect();

                let vertex_buffer = device.create_buffer_with_data(
                    bytemuck::cast_slice(&vertices),
                    BufferUsage::VERTEX
                );
                let index_buffer = device.create_buffer_with_data(
                    bytemuck::cast_slice(&model.mesh.indices),
                    BufferUsage::INDEX
                );

                Mesh {
                    name: model.name,
                    vertex_buffer,
                    index_buffer,
                    num_elements: model.mesh.indices.len() as u32,
                    material: model.mesh.material_id.unwrap_or(0),
                }
            }).collect();
        
            let instances = vec![Instance::default()];
            let instance_buffer = create_instance_buffer(&instances, device);

        Ok((Model { meshes, materials, instances, instance_buffer }, command_buffers))
    }

    pub fn new_light(device: &Device) -> Result<Self, failure::Error> {
        // Parse the `.obj` file. Optional is enabled to triangulate mesh.
        let (obj_models, obj_materials) = 
            tobj::load_obj("src/res/light.obj", true)?;

        // Iterate over the `tobj::Material` objects and convert them into 
        //    `crate::model::Material` objects with corresponding `wgpu::CommandBuffer` objects.
        let mut materials = Vec::new();

        // Iterate over the `tobj::Model` objects and convert them into `crate::model::Mesh` objects.
        let meshes: Vec<Mesh> = obj_models.into_iter()
            .map(|model| {
                let num_coords = model.mesh.positions.len() / 3;
                let vertices: Vec<ModelVertex> = (0..num_coords)
                    .map(|index| {
                        ModelVertex {
                            position: [
                                model.mesh.positions[index * 3],
                                model.mesh.positions[index * 3 + 1],
                                model.mesh.positions[index * 3 + 2],
                            ],
                            tex_coords: [
                                model.mesh.texcoords[index * 2],
                                model.mesh.texcoords[index * 2 + 1]
                            ],
                            normal: [
                                model.mesh.normals[index * 3],
                                model.mesh.normals[index * 3 + 1],
                                model.mesh.normals[index * 3 + 2],
                            ],
                        }
                    }).collect();

                let vertex_buffer = device.create_buffer_with_data(
                    bytemuck::cast_slice(&vertices),
                    BufferUsage::VERTEX
                );
                let index_buffer = device.create_buffer_with_data(
                    bytemuck::cast_slice(&model.mesh.indices),
                    BufferUsage::INDEX
                );

                Mesh {
                    name: model.name,
                    vertex_buffer,
                    index_buffer,
                    num_elements: model.mesh.indices.len() as u32,
                    material: model.mesh.material_id.unwrap_or(0),
                }
            }).collect();
        
            let instances = vec![Instance::default()];
            let instance_buffer = create_instance_buffer(&instances, device);

        Ok(Model { meshes, materials, instances, instance_buffer })
    }

    pub fn get_instance_buffer(&self) -> &wgpu::Buffer { &self.instance_buffer }
    pub fn set_instances(&mut self, instances: Vec<Instance>, device: &Device) {
        self.instances = instances;
        self.instance_buffer = create_instance_buffer(&self.instances, device);
    }
}

fn create_instance_buffer(instances: &Vec<Instance>, device: &Device) -> wgpu::Buffer {
    let instances_data: Vec<InstanceRaw> = 
        instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
    
    return device.create_buffer_with_data(
        bytemuck::cast_slice(&instances_data),
        wgpu::BufferUsage::VERTEX,
    );
}
