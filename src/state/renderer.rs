use wgpu::{RenderPass, RenderPipeline};
use crate::{
    light::Light,
    model::Model,
    uniforms::Uniforms,
};

/// An object used to render models to the screen.
pub struct Renderer {

    // The Models to be rendered.
    pub models: Vec<Model>,

    // The RenderPipeline object used to sent data to the GPU.
    render_pipeline: wgpu::RenderPipeline,

    // Whether the models are visibile, i.e. whether they should be rendered.
    pub visible: bool,
}

impl Renderer {

    /// Create a new Renderer object.
    ///
    /// By default visibility is enabled.
    ///
    /// # Arguments
    ///
    /// * `models`          - The Models to be rendered. 
    /// * `render_pipeline` - The RenderPipeline object used to sent data to the GPU.
    pub fn new(models: Vec<Model>, render_pipeline: RenderPipeline) -> Self {
        return Renderer { models, render_pipeline, visible: true }
    }

    /// Render the Models.
    ///
    /// # Arguments
    ///
    /// * `render_pass` - An object that connect RenderPipelines to the GPU.
    /// * `uniforms`    - The Uniforms objects needed by the shader progams.
    /// * `light`       - The Light object needed by the shader programs.
    pub fn render<'r>(
        &'r mut self,
        render_pass: &mut RenderPass<'r>,
        uniforms: &'r Uniforms,
        light: &'r Light,
    ) {
        if self.visible {
            render_pass.set_pipeline(&self.render_pipeline);
            for model in self.models.iter_mut() {
                use crate::model::DrawModel;
                render_pass.draw_model(model, &uniforms.bind_group, &light.bind_group);
            }
        }
    }
}