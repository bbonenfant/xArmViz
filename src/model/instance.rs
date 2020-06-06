use cgmath::{Matrix3, Matrix4, Vector3};
use wgpu::{BufferAddress, VertexBufferDescriptor};


/// Describes an instance of an object for the model.
pub struct Instance {

    // The position of the instance object.
    pub position: cgmath::Vector3<f32>,

    // The rotation of the instance object.
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    
    /// Construct an Instance object using a position vector.
    /// This is useful when the rotation of the instance does not matter.
    pub fn from_position(position: Vector3<f32>) -> Self {
        let rotation = {
            use cgmath::Quaternion;
            use cgmath::One;
            Quaternion::one()
        };
        return Instance { position, rotation }
    }

    /// Construct an InstanceRaw object from this object.
    pub fn to_raw(&self) -> InstanceRaw {
        let position_matrix = Matrix4::from_translation(self.position);
        let rotation_matrix = Matrix4::from(self.rotation);
        InstanceRaw::new(position_matrix * rotation_matrix)
    }
}

impl Default for Instance {
    fn default() -> Self {
        use cgmath::{One, Zero};
        return Instance {
            position: cgmath::Vector3::zero(),
            rotation: cgmath::Quaternion::one(),
        }
    }
}


/// The Raw data of an Instance that is sent to the GPU.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceRaw {

    // Matrix describing the position and rotation of the istance object.
    model: cgmath::Matrix4<f32>,

    // The Normal matrix. This is derived from the Model matrix.
    // The computation is done on the CPU as it is more efficient.
    normal: cgmath::Matrix3<f32>,
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

impl InstanceRaw {
    pub const SIZE: BufferAddress = std::mem::size_of::<InstanceRaw>() as BufferAddress;
    const FLOAT_SIZE: BufferAddress = std::mem::size_of::<f32>() as BufferAddress;
    const MODEL_SIZE: BufferAddress = Self::FLOAT_SIZE * 16;

    pub fn new(model: cgmath::Matrix4<f32>) -> Self {
        
        let normal = {
            use cgmath::SquareMatrix;
            let m = model.invert().expect("Matrix not invertible");
            // Do the transposition and conversion to 3x3 matrix in one step.
            Matrix3::new(
                m.x.x, m.y.x, m.z.x,
                m.x.y, m.y.y, m.z.y,
                m.x.z, m.y.z, m.z.z,
            )
        };
        return InstanceRaw { model, normal }
    }
}

impl super::Vertex for InstanceRaw {

    fn describe<'a>() -> VertexBufferDescriptor<'a> {
        return VertexBufferDescriptor {
            stride: Self::SIZE,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                // Describe the Model matrix (4x4).
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::FLOAT_SIZE * 4,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::FLOAT_SIZE * 4 * 2,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 5,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::FLOAT_SIZE * 4 * 3,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: 6,
                },
                // Describe the Normal matrix (3x3).
                wgpu::VertexAttributeDescriptor {
                    offset: Self::MODEL_SIZE,
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 7,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::MODEL_SIZE + (Self::FLOAT_SIZE * 3),
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 8,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: Self::MODEL_SIZE + (Self::FLOAT_SIZE * 3 * 2),
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 9,
                },
            ]
        }
    }
}