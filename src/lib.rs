use bevy::prelude::*;

pub mod prelude {
    pub use crate::{
        CameraAxisLimit, CameraFocus, CameraLimits, CameraMode, PrimaryCamera, SpookyCameraPlugin,
        Wrap,
    };
}

pub struct SpookyCameraPlugin;

impl Plugin for SpookyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraFocus::default())
            .add_systems(Update, (position_and_rotate_camera, update_camera_focus));
    }
}

#[derive(Resource, Default)]
pub struct CameraFocus {
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
}

impl CameraFocus {
    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn forward(&self) -> Vec3 {
        self.forward
    }

    pub fn right(&self) -> Vec3 {
        self.right
    }

    pub fn forward_flat(&self) -> Vec3 {
        let mut forward_vec = self.right;
        forward_vec.y = 0.0;
        forward_vec
    }

    pub fn right_flat(&self) -> Vec3 {
        let mut right_vec = self.right;
        right_vec.y = 0.0;
        right_vec
    }

    pub fn forward_randomized(&self, range: f32) -> Vec3 {
        let radius = (range * rand::random::<f32>()).sqrt();
        let theta = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
        let rot_x = (radius * theta.cos()).to_radians();
        let rot_y = (radius * theta.sin()).to_radians();
        let random_rotation = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), rot_y)
            .mul_quat(Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), rot_x));
        random_rotation.mul_vec3(self.forward)
    }
}

pub enum CameraAxisLimit {
    Clamp { min: f32, max: f32 },
    Wrap,
}

pub struct CameraLimits {
    pub x: CameraAxisLimit,
    pub y: CameraAxisLimit,
    pub z: CameraAxisLimit,
}

pub enum CameraMode {
    ThirdPersonOrbit,
    FirstPerson,
}

pub trait Wrap {
    fn wrap(&self) -> Self;
}

impl Wrap for f32 {
    fn wrap(&self) -> Self {
        if *self > 360.0 {
            return *self - 360.0;
        }
        if *self < 0.0 {
            return *self + 360.0;
        }
        *self
    }
}

#[derive(Component)]
pub struct PrimaryCamera {
    pub offset: Vec3,
    pub x_angle: f32,
    pub y_angle: f32,
    pub target: Vec3,
    pub mode: CameraMode,
    pub fov_degrees: f32,
    pub limits: CameraLimits,
}

impl PrimaryCamera {
    pub fn adjust_x_angle(&mut self, increase: f32) {
        match self.limits.x {
            CameraAxisLimit::Clamp { min, max } => {
                self.x_angle = (self.x_angle + increase).clamp(min, max);
            }
            CameraAxisLimit::Wrap => {
                self.x_angle = (self.x_angle + increase).wrap();
            }
        }
    }

    pub fn adjust_y_angle(&mut self, increase: f32) {
        self.y_angle += increase;
        match self.limits.y {
            CameraAxisLimit::Clamp { min, max } => {
                self.y_angle = (self.y_angle + increase).clamp(min, max);
            }
            CameraAxisLimit::Wrap => {
                self.y_angle = (self.y_angle + increase).wrap();
            }
        }
    }
}

impl Default for PrimaryCamera {
    fn default() -> Self {
        PrimaryCamera {
            offset: Vec3::new(0.0, 0.5, -6.0),
            x_angle: 0.0,
            y_angle: 0.0,
            target: Vec3::ZERO,
            mode: CameraMode::ThirdPersonOrbit,
            fov_degrees: 45.0,
            limits: CameraLimits {
                x: CameraAxisLimit::Clamp {
                    min: -2.0,
                    max: 20.0,
                },
                y: CameraAxisLimit::Wrap,
                z: CameraAxisLimit::Wrap,
            },
        }
    }
}

fn position_and_rotate_camera(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &PrimaryCamera)>,
) {
    if let Ok((mut transform, camera)) = camera_query.get_single_mut() {
        let mut starting_transform = Transform::from_translation(camera.target);
        let x_angle = camera.x_angle.to_radians();
        let y_angle = camera.y_angle.to_radians();

        starting_transform.rotate_y(y_angle);
        starting_transform.rotate_x(x_angle);

        let forward = starting_transform.forward().normalize();
        let right = starting_transform.right().normalize();

        let desired_position = match camera.mode {
            CameraMode::ThirdPersonOrbit => {
                starting_transform.translation
                    + (forward * camera.offset.z)
                    + (right * camera.offset.x)
                    + (Vec3::Y * camera.offset.y)
            }
            CameraMode::FirstPerson => starting_transform.translation + (Vec3::Y * camera.offset.y),
        };

        let mut desired_rotatation = Transform::default();

        desired_rotatation.rotate_x(x_angle);
        desired_rotatation.rotate_y(y_angle);

        let slerp_rotation = transform
            .rotation
            .slerp(desired_rotatation.rotation, time.delta_seconds() * 20.0);
        let lerp_position = transform
            .translation
            .lerp(desired_position, time.delta_seconds() * 20.0);

        transform.translation = lerp_position;
        transform.rotation = slerp_rotation;
    }
}

fn update_camera_focus(
    mut camera_focus: ResMut<CameraFocus>,
    camera_query: Query<&Transform, With<PrimaryCamera>>,
) {
    for transform in &camera_query {
        camera_focus.origin = transform.translation;
        camera_focus.forward = transform.forward();
        camera_focus.right = transform.right();
    }
}
