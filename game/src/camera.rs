use math::{NoE2Rotor, Rotor, Vector4};
use renderer::ray_tracing;
use std::{collections::HashSet, f32::consts::TAU};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vector4<f32>,
    pub base_rotation: NoE2Rotor,
    pub xy_rotation: f32,
}

impl Camera {
    pub fn rotation(&self) -> Rotor {
        Rotor::from_no_e2_rotor(self.base_rotation).then(Rotor::rotate_xy(self.xy_rotation))
    }

    pub fn update(&mut self, keys: &HashSet<KeyCode>, ts: f32) {
        let speed = 2.0;
        if keys.contains(&KeyCode::KeyW) {
            self.position += self.base_rotation.x() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyS) {
            self.position -= self.base_rotation.x() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyQ) {
            self.position -= self.base_rotation.y() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyE) {
            self.position += self.base_rotation.y() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyA) {
            self.position -= self.base_rotation.z() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyD) {
            self.position += self.base_rotation.z() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyR) {
            self.position += self.base_rotation.w() * speed * ts;
        }
        if keys.contains(&KeyCode::KeyF) {
            self.position -= self.base_rotation.w() * speed * ts;
        }

        let rotation_speed = 0.5 * TAU;
        if !keys.contains(&KeyCode::ShiftLeft) {
            if keys.contains(&KeyCode::ArrowUp) {
                self.xy_rotation += rotation_speed * ts;
            }
            if keys.contains(&KeyCode::ArrowDown) {
                self.xy_rotation -= rotation_speed * ts;
            }
            if keys.contains(&KeyCode::ArrowLeft) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xz(-rotation_speed * ts));
            }
            if keys.contains(&KeyCode::ArrowRight) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xz(rotation_speed * ts));
            }
        } else {
            if keys.contains(&KeyCode::ArrowUp) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xw(rotation_speed * ts));
            }
            if keys.contains(&KeyCode::ArrowDown) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_xw(-rotation_speed * ts));
            }
            if keys.contains(&KeyCode::ArrowLeft) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_zw(rotation_speed * ts));
            }
            if keys.contains(&KeyCode::ArrowRight) {
                self.base_rotation = self
                    .base_rotation
                    .then(NoE2Rotor::rotate_zw(-rotation_speed * ts));
            }
        }
    }

    pub fn into_render(&self, view: RenderView) -> ray_tracing::Camera {
        let rotation = self.rotation();
        let (forward, up, right) = match view {
            RenderView::XYZ => (rotation.x(), rotation.y(), rotation.z()),
            RenderView::XWZ => (rotation.x(), rotation.w(), rotation.z()),
            RenderView::XYW => (rotation.x(), rotation.y(), rotation.w()),
        };
        ray_tracing::Camera {
            position: self.position,
            forward,
            up,
            right,
            fov: 90f32.to_radians(),
            samples: 1,
            max_bounces: 4,
            random_seed: rand::random(),
        }
    }
}

pub enum RenderView {
    XYZ,
    XWZ,
    XYW,
}
