use crate::model::Model;
use super::LightSource;


pub struct Light {
    light_source: Box<dyn LightSource>,
    model: Model,
    pub visible: bool,
}

impl Light {
    pub fn new<L>(light_source: L, model: Model) -> Self 
      where L: LightSource + 'static {
        return Light { light_source: Box::new(light_source), model, visible: false }
    }

    pub fn get_model(&self) -> &Model { &self.model }
    pub fn get_buffer(&self) -> &wgpu::Buffer { &self.light_source.get_buffer() }
    pub fn get_bind_group(&self) -> &wgpu::BindGroup { &self.light_source.get_bind_group() }
}