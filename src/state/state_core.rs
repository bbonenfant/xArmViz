use winit::window::Window;
use wgpu::{DeviceDescriptor, SwapChainDescriptor};

type PhysicalSize = winit::dpi::PhysicalSize<u32>;


pub struct StateCore {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub swap_chain: wgpu::SwapChain,
    pub swap_chain_desc: wgpu::SwapChainDescriptor,
}

impl StateCore {

    const DEVICE_DESC: DeviceDescriptor = DeviceDescriptor {
        extensions: wgpu::Extensions { anisotropic_filtering: false },
        limits: wgpu::Limits { max_bind_groups: wgpu::MAX_BIND_GROUPS as u32 },
    };

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY, // Vulkan + Metal + DX12 + Browser WebGPU
        ).await.unwrap();

        let (device, queue) = adapter.request_device(&Self::DEVICE_DESC).await;

        let swap_chain_desc = create_swap_chain_desc(size);
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        return StateCore {
            adapter,
            device,
            queue,
            size,
            surface,
            swap_chain,
            swap_chain_desc,
        }
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        return (self.size.width as f32) / (self.size.height as f32)
    }

    pub fn resize(&mut self, new_size: PhysicalSize) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
    }

    pub fn submit(&self, command_buffers: &[wgpu::CommandBuffer]) {
        self.queue.submit(command_buffers)
    }
}


fn create_swap_chain_desc(size: PhysicalSize) -> SwapChainDescriptor {
    return SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    }
}