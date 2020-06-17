use std::path::Path;
use image::{DynamicImage, };
use wgpu::{
    BufferUsage,
    CommandBuffer,
    CommandEncoderDescriptor,
    Device,
    SwapChainDescriptor,
    TextureDescriptor,
};

const TEXTURE_BUFFER_COPY: CommandEncoderDescriptor = CommandEncoderDescriptor { label: Some("Texture Buffer Copy Encoder") };

type TextureResult = Result<(Texture, CommandBuffer), failure::Error>;


/// Structure for holding WPGU Texture objects.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    
    /// Load a texture from an image file.
    ///
    /// # Arguments
    ///
    /// * `device` - The connection to the graphics device. Used to create the rendering resources.
    /// * `path`   - The path to the image file.
    ///
    /// # Returns
    ///
    /// Result object that wraps a Tuple of (Texture, CommandBuffer).
    pub fn load<P: AsRef<Path>>(device: &Device, path: P) -> TextureResult {
        let path_copy = path.as_ref().to_path_buf();
        let label = path_copy.to_str();
        
        let img = image::open(path)?;
        return Self::from_image(device, &img, label)
    }

    /// Load a Texture from an bytes image.
    ///
    /// # Arguments
    ///
    /// * `device` - The connection to the graphics device. Used to create the rendering resources.
    /// * `bytes` - The images as bytes.
    ///
    /// # Returns
    ///
    /// Result object that wraps a Tuple of (Texture, CommandBuffer).
    pub fn from_bytes(device: &Device, bytes: &[u8], label: &str) -> TextureResult {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, &img, Some(label))
    }

    /// Create a texture using a single color.
    ///
    /// # Arguments
    ///
    /// * `device` - The connection to the graphics device. Used to create the rendering resources.
    /// * `color`  - The color to use in the texture.
    ///
    /// # Returns
    ///
    /// Result object that wraps a Tuple of (Texture, CommandBuffer).
    pub fn from_color(device: &Device, color: image::Rgba<u8>) -> TextureResult {
        use image::GenericImage;
        let mut img = DynamicImage::new_rgba8(1, 1);
        img.put_pixel(0, 0, color);
       
        return Self::from_image(device, &img, Some("Color Texture"))
    }

    /// Create a texture using a single random color.
    ///
    /// # Arguments
    ///
    /// * `device` - The connection to the graphics device. Used to create the rendering resources.
    ///
    /// # Returns
    ///
    /// Result object that wraps a Tuple of (Texture, CommandBuffer).
    pub fn from_random_color(device: &Device) -> TextureResult {
        use rand::random;
        let color = [random::<u8>(), random::<u8>(), random::<u8>(), 0];
        return Self::from_color(device, color.into())
    }

    /// Creates a Texture from a DynamicImage object.
    ///
    /// # Arguments
    ///
    /// * `device` - The connection to the graphics device. Used to create the rendering resources.
    /// * `img` - The image parsed into a DynamicImage object.
    ///
    /// # Returns
    ///
    /// Result object that wraps a Tuple of (Texture, CommandBuffer).
    pub fn from_image(device: &Device, img: &DynamicImage, label: Option<&str>) -> TextureResult {
        let rgba = img.to_rgba();
        let dimensions = {
            use image::GenericImageView;
            img.dimensions()
        };

        let size = wgpu::Extent3d { width: dimensions.0, height: dimensions.1, depth: 1 };
        let texture = device.create_texture(
            &TextureDescriptor {
                label,
                size,
                array_layer_count: 1,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            }
        );

        let cmd_buffer = {
            let mut encoder = device.create_command_encoder(&TEXTURE_BUFFER_COPY);
            let buffer = device.create_buffer_with_data(&rgba, BufferUsage::COPY_SRC);
            encoder.copy_buffer_to_texture(
                wgpu::BufferCopyView {
                    buffer: &buffer,
                    offset: 0,
                    bytes_per_row: 4 * dimensions.0,
                    rows_per_image: dimensions.1,
                },
                wgpu::TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                size,
            );
    
            encoder.finish()
        };

        let view = texture.create_default_view();
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: wgpu::CompareFunction::Always,
            }
        );

        Ok((Self { texture, view, sampler }, cmd_buffer))
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; 
    
    /// Create a depth texture for the screen.
    ///
    /// # Arguments
    ///
    /// * `device`          - The connection to the graphics device.
    ///                       Used to create the rendering resources.
    /// * `swap_chain_desc` - Descriptor for the swap chain.
    /// * `label`           - The label for the TextureDescriptor.
    ///
    /// # Returns
    ///
    /// Result object that wraps a Tuple of (Texture, CommandBuffer).
    pub fn create_depth_texture(device: &Device, swap_chain_desc: &SwapChainDescriptor, label: &str) -> Self {
        let size = wgpu::Extent3d { 
            width: swap_chain_desc.width,
            height: swap_chain_desc.height,
            depth: 1 
        };
        let desc = TextureDescriptor {
            label: Some(label),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: (
                wgpu::TextureUsage::COPY_SRC
              | wgpu::TextureUsage::OUTPUT_ATTACHMENT
              | wgpu::TextureUsage::SAMPLED
            ),
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_default_view();
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: wgpu::CompareFunction::LessEqual,
            }
        );

        Self { texture, view, sampler }
    }

    pub fn create_shadow_texture(device: &Device, swap_chain_desc: &SwapChainDescriptor, label: &str) -> Self {
        let size = wgpu::Extent3d { 
            width: swap_chain_desc.width,
            height: swap_chain_desc.height,
            depth: 1 
        };
        let desc = TextureDescriptor {
            label: Some(label),
            size,
            array_layer_count: crate::light::Lighting::MAX_LIGHTS as u32,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: (
                wgpu::TextureUsage::COPY_SRC
              | wgpu::TextureUsage::OUTPUT_ATTACHMENT
              | wgpu::TextureUsage::SAMPLED
            ),
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_default_view();
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: wgpu::CompareFunction::LessEqual,
            }
        );

        Self { texture, view, sampler }
    }    
}
