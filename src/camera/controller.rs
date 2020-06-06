use cgmath::Deg;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use super::Camera;


/// Holds information about whether Camera controlling keys are being pressed.
pub struct CameraController {

    is_up_pressed: bool,
    is_down_pressed: bool,

    is_left_pressed: bool,
    is_right_pressed: bool,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    
    is_cw_pressed: bool,
    is_ccw_pressed: bool,
}

impl CameraController {

    /// Creates a new CameraController.
    pub fn new() -> Self {
        Self {
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_cw_pressed: false,
            is_ccw_pressed: false,
        }
    }

    /// Process a WindowEvent.
    /// Returns whether any event was processed.
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput { state, virtual_keycode: Some(keycode), .. },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up    => { self.is_up_pressed       = is_pressed; }
                    VirtualKeyCode::S | VirtualKeyCode::Down  => { self.is_down_pressed     = is_pressed; }

                    VirtualKeyCode::A | VirtualKeyCode::Left  => { self.is_left_pressed     = is_pressed; }
                    VirtualKeyCode::D | VirtualKeyCode::Right => { self.is_right_pressed    = is_pressed; }

                    VirtualKeyCode::LShift                    => { self.is_forward_pressed  = is_pressed; }
                    VirtualKeyCode::LControl                  => { self.is_backward_pressed = is_pressed; }
                    
                    VirtualKeyCode::E                         => { self.is_cw_pressed       = is_pressed; }
                    VirtualKeyCode::Q                         => { self.is_ccw_pressed      = is_pressed; }

                    _ => return false, // If some other Key was pressed or released.
                }
            }
            _ => return false, // If the event was not a KeyboardInput event.
        }
        // This statement is reached only if a Camera controlling KevboardInput event occurred.
        return true
    }

    /// Update the Camera position and rotation based upon the current state of the CamerController.
    /// Returns whether the Camera was updated.
    pub fn update_camera(&self, camera: &mut Camera) -> bool {
        const SPEED: f32 = 0.3;
        const THETA: Deg<f32> = cgmath::Deg(6.0);
        const ZERO: Deg<f32> = cgmath::Deg(0.0);

        let yaw = 
            if self.is_right_pressed   { THETA } else if self.is_left_pressed     { -THETA } else { ZERO };
        let pitch = 
            if self.is_up_pressed      { THETA } else if self.is_down_pressed     { -THETA } else { ZERO };
        let roll = 
            if self.is_ccw_pressed     { THETA } else if self.is_cw_pressed       { -THETA } else { ZERO };
        let radial =
            if self.is_forward_pressed { SPEED } else if self.is_backward_pressed { -SPEED } else { 0f32 };
        
        // If nothing changed, don't perform any calculations.
        if (yaw == ZERO) && (pitch == ZERO) && (roll == ZERO) && (radial == 0f32) {
            return false
        }

        camera.set_view(
            camera.get_view().spherical_adjust(yaw, pitch, roll, radial)
        );
        return true
    }
}