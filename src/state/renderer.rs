use wgpu::{RenderPass, RenderPipeline};
use crate::{
    light::{Lighting, Spotlight},
    model::Model,
    texture::Texture,
    uniforms::Uniforms,
};

/// An object used to render models to the screen.
pub struct Renderer {

    // The Models to be rendered.
    pub models: Vec<Model>,

    // The RenderPipeline object used to sent data to the GPU.
    render_pipeline: wgpu::RenderPipeline,

    // The Pipeline used to construct the shadow depth map.
    shadow_pipeline: Option<wgpu::RenderPipeline>,

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
    pub fn new(
        models: Vec<Model>,
        render_pipeline: RenderPipeline,
        shadow_pipeline: Option<RenderPipeline>
    ) -> Self {
        return Renderer { models, render_pipeline, shadow_pipeline, visible: true }
    }

    pub fn construct_texture<'t>(
        &'t mut self,
        render_pass: &mut RenderPass<'t>,
        light: &'t Spotlight
    ) {
        if self.visible {
            let shadow_pipeline = self.shadow_pipeline.as_ref().expect("No shadow pipeline exists.");
            render_pass.set_pipeline(&shadow_pipeline);
            for model in self.models.iter_mut() {
                use crate::model::ConstructShadowMap;
                render_pass.construct_model_shadow(model, &light.bind_group);
            }
        }
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
        lighting: &'r Lighting,
        shadow_bind_group: &'r wgpu::BindGroup,
    ) {
        if self.visible {
            render_pass.set_pipeline(&self.render_pipeline);
            for model in self.models.iter_mut() {
                use crate::model::DrawModel;
                render_pass.draw_model(model, &uniforms.bind_group, &lighting.get_bind_group(), shadow_bind_group);
            }
        }
    }
}