use shaderc::ShaderKind;
use std::io::Cursor;

pub struct ShaderData {
    pub fragment: Option<Vec<u32>>,
    pub vertex: Vec<u32>,
}


lazy_static! {
    pub static ref MODEL_SHADER_DATA: ShaderData = 
        ShaderData {
            fragment: Some({
                let mut compiler = shaderc::Compiler::new().unwrap();
                let spirv = compiler.compile_into_spirv(
                    include_str!("src/model.frag"),
                    ShaderKind::Fragment,
                    "model.frag",
                    "main",
                    None,
                ).unwrap();
                wgpu::read_spirv(Cursor::new(spirv.as_binary_u8())).unwrap()
            }),
            vertex: {
                let mut compiler = shaderc::Compiler::new().unwrap();
                let spirv = compiler.compile_into_spirv(
                    include_str!("src/model.vert"),
                    ShaderKind::Vertex,
                    "model.vert",
                    "main",
                    None,
                ).unwrap();
                wgpu::read_spirv(Cursor::new(spirv.as_binary_u8())).unwrap()
            },
        };
    
    pub static ref LIGHT_SHADER_DATA: ShaderData = 
        ShaderData {
            fragment: Some({
                let mut compiler = shaderc::Compiler::new().unwrap();
                let spirv = compiler.compile_into_spirv(
                    include_str!("src/light.frag"),
                    ShaderKind::Fragment,
                    "light.frag",
                    "main",
                    None,
                ).unwrap();
                wgpu::read_spirv(Cursor::new(spirv.as_binary_u8())).unwrap()
            }),
            vertex: {
                let mut compiler = shaderc::Compiler::new().unwrap();
                let spirv = compiler.compile_into_spirv(
                    include_str!("src/light.vert"),
                    ShaderKind::Vertex,
                    "light.vert",
                    "main",
                    None,
                ).unwrap();
                wgpu::read_spirv(Cursor::new(spirv.as_binary_u8())).unwrap()
            },
        };
    
    pub static ref SHADOW_SHADER_DATA: ShaderData = 
        ShaderData {
            fragment: None,
            vertex: {
                let mut compiler = shaderc::Compiler::new().unwrap();
                let spirv = compiler.compile_into_spirv(
                    include_str!("src/shadow.vert"),
                    ShaderKind::Vertex,
                    "shadow.vert",
                    "main",
                    None,
                ).unwrap();
                wgpu::read_spirv(Cursor::new(spirv.as_binary_u8())).unwrap()
            },
        };
}