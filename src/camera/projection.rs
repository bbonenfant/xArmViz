use cgmath::{Deg, Matrix4};

/// Representation of a 3D Projection onto a 2D Screen.
#[derive(Clone, Copy)]
pub struct Projection {

    // The aspect ratio of the screen.
    pub aspect: f32,

    // The vertical Field of View (in degrees)
    pub fov_y: Deg<f32>,

    // The minimum distance that is projected.
    pub z_near: f32,

    // The maximum distance that is projected.
    pub z_far: f32,

    // The projection matrix. This is cached.
    projection: Matrix4<f32>,
}


impl Projection {

    pub const DEFAULT_FOV_Y: cgmath::Deg<f32> = Deg(45.0);
    pub const DEFAULT_Z_NEAR: f32 = 0.1;
    pub const DEFAULT_Z_FAR: f32 = 100.0;

    /// Create a new Projection object. Constructs the Projection matrix at creation.
    pub fn new(aspect: f32, fov_y: Deg<f32>, z_near: f32, z_far: f32) -> Self {
        let projection = cgmath::perspective(fov_y, aspect, z_near, z_far);
        return Projection { aspect, fov_y, z_near, z_far, projection }
    }

    pub fn with_aspect(aspect: f32) -> Self {
        return Projection::new(
            aspect, 
            Self::DEFAULT_FOV_Y,
            Self::DEFAULT_Z_NEAR,
            Self::DEFAULT_Z_FAR
        )
    }

    /// Getter for the Projection Matrix.
    pub fn as_matrix(&self) -> Matrix4<f32> { self.projection }
}
