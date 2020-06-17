use cgmath::{
    Basis3,
    Deg,
    Matrix4,
    Point3,
    Rotation3,
    Vector3
};
use super::Projection;

/// Representation of the View.
///   An encoding of the position and orientation of the viewer.
#[derive(Clone, Copy)]
pub struct View {

    // The position of the viewer.
    eye: Point3<f32>,

    // The target that the viewer is looking at.
    target: Point3<f32>,

    // The Vector pointing up (with respect to the viewer).
    up: Vector3<f32>,

    // The View Matrix. This is cached.
    view: Matrix4<f32>,
}

impl View {

    /// Construct a new View. Checks orthogonality of Up Vector.
    pub fn new(eye: Point3<f32>, target: Point3<f32>, up: Vector3<f32>) -> Self {
        use cgmath::InnerSpace;
        let up = up.normalize();
        let view = Matrix4::look_at(eye, target, up);
        return View { eye, target, up, view }
    }

    pub fn get_position(&self) -> Point3<f32> { self.eye }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.eye = position;
        self.view = Matrix4::look_at(self.eye, self.target, self.up);
    }
    
    /// Getter for the View Matrix.
    pub fn as_matrix(&self) -> Matrix4<f32> { self.view }

    /// Creates a new View object based on spherical adjustments to the viewer's orientation and radial position.
    ///   The target is preserved in the new View.
    ///
    /// # Arguments
    /// 
    /// * `yaw` - The change in the Yaw angle, in degrees.
    /// * `pitch` - The change in the Pitch angle, in degrees.
    /// * `roll` - The change in the Roll angle, in degrees.
    /// * `radial` - The change in radial distance from the target.
    pub fn spherical_adjust(self, yaw: Deg<f32>, pitch: Deg<f32>, roll: Deg<f32>, radial: f32) -> Self {
        use cgmath::{EuclideanSpace, InnerSpace, Rotation};

        // Construct the Rotation Matrices.
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up);
        let yaw_rot: Basis3<f32> = Rotation3::from_axis_angle(self.up, yaw);
        let pitch_rot: Basis3<f32> = Rotation3::from_axis_angle(right, pitch);
        let roll_rot: Basis3<f32> = Rotation3::from_axis_angle(forward, roll);
        
        // Apply Transformations.
        let eye_vec = self.eye.to_vec();
        let magnitude = {
            match eye_vec.magnitude() - radial {
                x if x > Projection::DEFAULT_Z_NEAR => x,
                _ => Projection::DEFAULT_Z_NEAR
            }
        };
        let eye = Point3::<f32>::from_vec(
            yaw_rot.rotate_vector(
                pitch_rot.rotate_vector(
                    roll_rot.rotate_vector(eye_vec)
                )
            ).normalize_to(magnitude)
        );
        let up = {
            let up = 
                yaw_rot.rotate_vector(
                    pitch_rot.rotate_vector(
                        roll_rot.rotate_vector(self.up)
                    )
                ).normalize();
            // Use Gram-Schmidt method to correct error in orthogonality. 
            let correction = (forward.dot(up) / forward.dot(forward)) * forward;
            up - correction
        };

        // Construct new View.
        return View::new(eye, self.target, up)
    }
}


pub const DEFAULT_EYE: [f32; 3] = [0.0, 0.0, 50.0];
pub const DEFAULT_TARGET: [f32; 3] = [0.0, 0.0, 0.0];
pub const DEFAULT_UP: [f32; 3] = [0.0, 1.0, 0.0];

impl Default for View {

    fn default() -> Self {
        View::new(
            DEFAULT_EYE.into(),
            DEFAULT_TARGET.into(),
            DEFAULT_UP.into(),
        )
    }
}