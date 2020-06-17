mod baker;
mod light;
mod lighting;
mod raw;
mod spotlight;
mod traits;

pub use baker::ShadowBaker;
pub use light::Light;
pub use lighting::Lighting;
pub use raw::LightRaw;
pub use spotlight::Spotlight;
pub use traits::LightSource;


const BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor = {
    const VISIBILITY: wgpu::ShaderStage = wgpu::ShaderStage::from_bits_truncate(
        wgpu::ShaderStage::VERTEX.bits() | wgpu::ShaderStage::FRAGMENT.bits()
    );
    wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: VISIBILITY,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: VISIBILITY,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: None,
    }
};