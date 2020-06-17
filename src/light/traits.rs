use super::LightRaw;

pub trait LightSource {
    fn as_light_raw(&self) -> LightRaw;

    fn get_buffer(&self) -> &wgpu::Buffer;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}